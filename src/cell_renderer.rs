use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    math::prelude::*,
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::ExtractedView,
        RenderApp, RenderStage,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::utils;

#[derive(Component)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);
impl ExtractComponent for InstanceMaterialData {
    type Filter = ();
    type Query = &'static InstanceMaterialData;

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        InstanceMaterialData(item.0.clone())
    }
}

pub struct CellMaterialPlugin;
impl Plugin for CellMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<InstanceMaterialData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CellPipeline>()
            .init_resource::<SpecializedMeshPipelines<CellPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CellPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CellPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<
        (Entity, &MeshUniform, &Handle<Mesh>),
        (With<Handle<Mesh>>, With<InstanceMaterialData>),
    >,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .expect("DrawCustom not found");

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, mesh_uniform, mesh_handle) in material_meshes.iter() {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    msaa_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .expect("Pipeline not found");
                transparent_phase.add(Transparent3d {
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                    pipeline,
                    entity,
                    draw_function: draw_custom,
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.0.len(),
        });
    }
}

pub struct CellPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for CellPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world
            .get_resource::<AssetServer>()
            .expect("AssetServer not found");
        asset_server
            .watch_for_changes()
            .expect("Watching for changes failed");
        let shader = asset_server.load("shaders/cell.wgsl");

        let mesh_pipeline = world
            .get_resource::<MeshPipeline>()
            .expect("MeshPipeline not found");

        CellPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for CellPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal, and UV
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor
            .fragment
            .as_mut()
            .expect("Fragment shader not found")
            .shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
        ]);

        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshedInstanced,
);

pub struct DrawMeshedInstanced;
impl EntityRenderCommand for DrawMeshedInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<InstanceBuffer>>,
    );

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, instance_buffer_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).expect("missing mesh");
        let instance_buffer = instance_buffer_query
            .get(item)
            .expect("missing instance buffer");

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }

        RenderCommandResult::Success
    }
}

pub struct CellRenderer {
    pub bounds: i32,
    pub values: Vec<u8>,
    pub neighbors: Vec<u8>,
}

impl CellRenderer {
    pub fn new() -> Self {
        Self {
            bounds: 0,
            values: vec![],
            neighbors: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.values.truncate(0);
        self.values.resize(self.cell_count(), 0);
        self.neighbors.truncate(0);
        self.neighbors.resize(self.cell_count(), 0);
    }

    pub fn cell_count(&self) -> usize {
        self.bounds.pow(3) as usize
    }

    pub fn set_bounds(&mut self, new_bounds: i32) {
        if new_bounds != self.bounds {
            let new_count = new_bounds.pow(3);
            self.values.resize(new_count as usize, 0);
            self.neighbors.resize(new_count as usize, 0);
            self.bounds = new_bounds;
        }
    }

    pub fn set(&mut self, idx: usize, value: u8, neighbors: u8) {
        self.values[idx] = value;
        self.neighbors[idx] = neighbors;
    }

    pub fn set_pos(&mut self, pos: IVec3, value: u8, neigbors: u8) {
        self.set(utils::pos_to_idx(pos, self.bounds), value, neigbors);
    }
}
