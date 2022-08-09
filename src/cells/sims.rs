use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_egui::{egui, EguiContext};

use crate::{
    cell_renderer::{CellRenderer, InstanceData, InstanceMaterialData},
    cells::Sim,
    color_method::ColorMethod,
    neighbors::NeighborMethod,
    rule::Rule,
    utils,
};

#[derive(Clone)]
pub struct Example {
    pub name: String,
    pub rule: Rule,
    pub color_method: ColorMethod,
    pub color1: Color,
    pub color2: Color,
}

pub struct Sims {
    sims: Vec<(String, Box<dyn Sim>)>,
    active_sim: usize,
    bounds: i32,
    update_dt: std::time::Duration,

    renderer: Option<Box<CellRenderer>>,

    rule: Option<Rule>, // dumb, maybe use cell
    color_method: ColorMethod,
    color1: Color,
    color2: Color,

    examples: Vec<Example>,
}

impl Sims {
    pub fn new() -> Self {
        Self {
            sims: vec![],
            active_sim: usize::MAX,
            bounds: 64,
            update_dt: std::time::Duration::from_secs(0),
            renderer: Some(Box::new(CellRenderer::new())),
            rule: None,
            color_method: ColorMethod::DistToCenter,
            color1: Color::YELLOW,
            color2: Color::RED,
            examples: vec![],
        }
    }

    pub fn add_sim(&mut self, name: String, sim: Box<dyn Sim>) {
        self.sims.push((name, sim));
    }

    pub fn add_example(&mut self, example: Example) {
        self.examples.push(example);
    }

    pub fn set_sim(&mut self, idx: usize) {
        if self.active_sim < self.sims.len() {
            self.sims[self.active_sim].1.reset();
        }

        let rule = self.rule.take().expect("taking rule");
        self.active_sim = idx;
        self.bounds = self.sims[idx].1.set_bounds(self.bounds);
        self.sims[idx].1.spawn_noise(&rule);
        self.renderer
            .as_mut()
            .expect("get renderer")
            .set_bounds(self.bounds);
        self.rule = Some(rule);
    }

    pub fn set_example(&mut self, idx: usize) {
        let example = self.examples[idx].clone();
        let rule = example.rule;
        self.color_method = example.color_method;
        self.color1 = example.color1;
        self.color2 = example.color2;

        if self.active_sim < self.sims.len() {
            let sim = &mut self.sims[self.active_sim].1;
            sim.reset();
            sim.spawn_noise(&rule);
        }
        self.rule = Some(rule);
    }
}

pub fn update(
    mut this: ResMut<Sims>,
    mut query: Query<&mut InstanceMaterialData>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if this.active_sim > this.sims.len() {
        this.set_sim(0);
    }

    let mut bounds = this.bounds;
    let mut active_sim = this.active_sim;

    egui::Window::new("Celluar!").show(egui_ctx.ctx_mut(), |ui| {
        let old_bounds = bounds;
        let old_active = active_sim;

        ui.label("Simulator:");
        {
            egui::ComboBox::from_id_source("simulator")
                .selected_text(&this.sims[active_sim].0)
                .show_ui(ui, |ui| {
                    for (i, (name, _)) in this.sims.iter().enumerate() {
                        ui.selectable_value(&mut active_sim, i, name);
                    }
                });

            if active_sim != old_active {
                this.set_sim(active_sim);
                bounds = this.bounds; // he doesn't like it.
            }

            let update_dt = this.update_dt;
            let rule = this.rule.take().expect("taking rule");
            let sim = &mut this.sims[active_sim].1;

            let cell_count = sim.cell_count();
            ui.label(format!("cells: {}", cell_count));
            ui.label(format!(
                "update: {:.2?} per cell",
                update_dt / cell_count.max(1) as u32
            ));

            if ui.button("reset").clicked() {
                sim.reset();
            }

            if ui.button("spawn noise").clicked() {
                sim.spawn_noise(&rule);
            }

            ui.add(egui::Slider::new(&mut bounds, 32..=128).text("bounding size"));
            if bounds != old_bounds {
                bounds = sim.set_bounds(bounds);
                sim.spawn_noise(&rule);
                this.renderer
                    .as_mut()
                    .expect("get renderer")
                    .set_bounds(bounds);
            }

            this.rule = Some(rule);
        }

        ui.add_space(24.0);
        ui.label("Rules:");
        {
            egui::ComboBox::from_label("color method")
                .selected_text(format!("{:?}", this.color_method))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut this.color_method, ColorMethod::Single, "Single");
                    ui.selectable_value(
                        &mut this.color_method,
                        ColorMethod::StateLerp,
                        "State Lerp",
                    );
                    ui.selectable_value(
                        &mut this.color_method,
                        ColorMethod::DistToCenter,
                        "Distance to Center",
                    );
                    ui.selectable_value(&mut this.color_method, ColorMethod::Neighbor, "Neighbor");
                });

            color_picker(ui, &mut this.color1);
            color_picker(ui, &mut this.color2);

            let mut rule = this.rule.take().expect("taking rule");
            let old_rule = rule.clone();

            egui::ComboBox::from_label("Neigbor method")
                .selected_text(format!("{:?}", rule.neighbor_method))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut rule.neighbor_method, NeighborMethod::Moore, "Moore");
                    ui.selectable_value(
                        &mut rule.neighbor_method,
                        NeighborMethod::VonNeumann,
                        "Von Neumann",
                    );
                });

            ui.add(egui::Slider::new(&mut rule.states, 1..=50).text("states"));

            // TODO: survival & birth rule;

            if rule != old_rule {
                let sim = &mut this.sims[active_sim].1;
                sim.reset();
                sim.spawn_noise(&rule);
            }

            this.rule = Some(rule)
        }

        ui.add_space(24.0);

        ui.label("Examples:");
        for i in 0..this.examples.len() {
            let example = &this.examples[i];
            if ui.button(&example.name).clicked() {
                this.set_example(i);
            }
        }
    });

    let rule = this.rule.take().expect("taking rule");
    let mut renderer = this.renderer.take().expect("taking renderer");

    let sim = &mut this.sims[active_sim].1;

    let t0 = std::time::Instant::now();
    let task_pool = AsyncComputeTaskPool::get();
    sim.update(&rule, task_pool);
    let update_dt = t0.elapsed();

    sim.render(&mut renderer);

    let instance_data = &mut query.iter_mut().next().expect("getting instance data").0;
    instance_data.truncate(0);
    for idx in 0..renderer.cell_count() {
        let value = renderer.values[idx];
        let neighbors = renderer.neighbors[idx];

        if value != 0 {
            let pos = utils::idx_to_pos(idx, bounds);
            instance_data.push(InstanceData {
                position: (pos - utils::center(bounds)).as_vec3(),
                scale: 1.0,
                color: this
                    .color_method
                    .color(
                        this.color2,
                        this.color2,
                        rule.states,
                        value,
                        neighbors,
                        utils::dist_to_center(pos, bounds),
                    )
                    .into(),
            });
        }
    }

    this.bounds = bounds;
    this.active_sim = active_sim;
    this.update_dt = update_dt;
    this.renderer = Some(renderer);
    this.rule = Some(rule);
}

pub struct SimsPlugin;
impl Plugin for SimsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sims::new()).add_system(update);
    }
}

pub fn color_picker(ui: &mut egui::Ui, color: &mut Color) {
    let mut c = [
        (color.r() * 255.0) as u8,
        (color.g() * 255.0) as u8,
        (color.b() * 255.0) as u8,
    ];
    egui::color_picker::color_edit_button_srgb(ui, &mut c);
    color.set_r(c[0] as f32 / 255.0);
    color.set_g(c[1] as f32 / 255.0);
    color.set_b(c[2] as f32 / 255.0);
}
