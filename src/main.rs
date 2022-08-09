use bevy::{prelude::*, render::view::NoFrustumCulling, tasks::{AsyncComputeTaskPool, TaskPoolBuilder}};
use bevy_egui::EguiPlugin;

mod cells;
use cells::Example;

mod cell_event;
use cell_event::CellStatesChangedEvent;

mod cell_renderer;
use cell_renderer::{CellMaterialPlugin, InstanceData, InstanceMaterialData};

mod color_method;

mod neighbors;
use color_method::ColorMethod;
use neighbors::NeighborMethod;

mod rotating_camera;
use rotating_camera::{RotatingCamera, RotatingCameraPlugin};

mod rule;
use rule::{Rule, Value};

mod utils;

fn main() {
    let mut task_pool_settings = DefaultTaskPoolOptions::default();
    task_pool_settings.async_compute.percent = 1.0f32;
    task_pool_settings.compute.percent = 0.0f32;
    task_pool_settings.io.percent = 0.0f32;

    App::new()
        .insert_resource(task_pool_settings)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(ClearColor(Color::rgb(0.65f32, 0.9f32, 0.96f32)))
        .add_event::<CellStatesChangedEvent>()
        .add_plugin(RotatingCameraPlugin)
        .add_plugin(CellMaterialPlugin)
        .add_plugin(cells::SimsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut sims: ResMut<cells::Sims>) {
    sims.add_sim(
        "tantan single-threaded".into(),
        Box::new(cells::tantan::CellsSingleThreaded::new()),
    );

    sims.add_example(Example {
        name: "builder".into(),
        rule: Rule {
            survival_rule: Value::new(&[2, 6, 9]),
            birth_rule: Value::new(&[4, 6, 8, 9, 10]),
            states: 10,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: color_method::ColorMethod::DistToCenter,
        color1: Color::YELLOW,
        color2: Color::RED,
    });

    sims.add_example(Example {
        name: "VN pyramid".into(),
        rule: Rule {
            survival_rule: Value::from_range(0..=6),
            birth_rule: Value::new(&[1,3]),
            states: 2,
            neighbor_method: NeighborMethod::VonNeumann,
        },
        color_method: ColorMethod::DistToCenter,
        color1: Color::GREEN,
        color2: Color::BLUE,
    });

    sims.add_example(Example {
        name: "fancy snancy".into(),
        rule: Rule {
            survival_rule: Value::new(&[0,1,2,3,7,8,9,11,13,18,21,22,24,26]),
            birth_rule: Value::new(&[4,13,17,20,21,22,23,24,26]),
            states: 4,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::RED,
        color2: Color::BLUE,
    });

    sims.add_example(Example {
        name: "pretty crystals".into(),
        rule: Rule {
            survival_rule: Value::new(&[5,6,7,8]),
            birth_rule: Value::new(&[6,7,9]),
            states: 10,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::DistToCenter,
        color1: Color::GREEN,
        color2: Color::BLUE,
    });

    sims.add_example(Example {
        name: "swapping structures".into(),
        rule: Rule {
            survival_rule: Value::new(&[3,6,9]),
            birth_rule: Value::new(&[4,8,10]),
            states: 20,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::RED,
        color2: Color::GREEN,
    });

    sims.add_example(Example {
        name: "slowly expanding blob".into(),
        rule: Rule {
            survival_rule: Value::from_range(9..=26),
            birth_rule: Value::new(&[5,6,7,12,13,15]),
            states: 20,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::YELLOW,
        color2: Color::BLUE,
    });

    sims.add_example(Example {
        name: "445".into(),
        rule: Rule {
            survival_rule: Value::new(&[4]),
            birth_rule: Value::new(&[4]),
            states: 5,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::BLACK,
        color2: Color::RED,
    });

    sims.add_example(Example {
        name: "expand then die".into(),
        rule: Rule {
            survival_rule: Value::new(&[4]),
            birth_rule: Value::new(&[3]),
            states: 20,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::BLACK,
        color2: Color::RED,
    });

    sims.add_example(Example {
        name: "no idea what to call this".into(),
        rule: Rule {
            survival_rule: Value::new(&[6,7]),
            birth_rule: Value::new(&[4,6,9,10,11]),
            states: 6,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::BLUE,
        color2: Color::RED,
    });

    sims.add_example(Example {
        name: "LARGE LINES".into(),
        rule: Rule {
            survival_rule: Value::new(&[5]),
            birth_rule: Value::new(&[4, 6, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23, 24]),
            states: 35,
            neighbor_method: NeighborMethod::Moore,
        },
        color_method: ColorMethod::StateLerp,
        color1: Color::BLUE,
        color2: Color::RED,
    });

    sims.set_example(0);

    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        InstanceMaterialData(
            (1..=10)
                .flat_map(|x| (1..=10).map(move |y| (x as f32 / 10.0, y as f32 / 10.0)))
                .map(|(x, y)| InstanceData {
                    position: Vec3::new(x * 10.0 - 5.0, y * 10.0 - 5.0, 0.0),
                    scale: 1.0,
                    color: Color::hsla(x * 360., y, 0.5, 1.0).as_rgba_f32(),
                })
                .collect(),
        ),
        Visibility::default(),
        ComputedVisibility::default(),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        NoFrustumCulling,
    ));

    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(RotatingCamera::default());
}
