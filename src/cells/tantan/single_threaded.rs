use std::collections::HashMap;

use bevy::prelude::*;

use crate::{cells::Sim, rule::Rule, cell_renderer::CellRenderer, utils};

use super::CellState;

pub struct CellsSingleThreaded {
    states: HashMap<IVec3, CellState>,
    bounding_size: i32,
    neighbors: HashMap<IVec3, u8>,
    changes: HashMap<IVec3, i32>,
    spawn: Vec<(IVec3, u8)>,
}

impl CellsSingleThreaded {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            bounding_size: 0,
            neighbors: HashMap::new(),
            changes: HashMap::new(),
            spawn: Vec::new(),
        }
    }

    pub fn tick(&mut self, rule: &Rule) {

    }
}

impl Sim for CellsSingleThreaded {
    fn update(&mut self, rule: &Rule, task_pool: &bevy::tasks::TaskPool) {
        self.tick(rule);
    }

    fn render(&self, renderer: &mut CellRenderer) {
        renderer.clear();
        for cell in self.states.iter() {
            renderer.set_pos(*cell.0, cell.1.value, cell.1.neighbors);
        }
    }

    fn spawn_noise(&mut self, rule: &Rule) {
        utils::make_some_noise_default(utils::center(self.bounding_size), |pos| {
            self.states.insert(pos, CellState::new(rule.states, 0));
        })
    }

    fn cell_count(&self) -> usize {
        self.states.len()
    }

    fn bounds(&self) -> i32 {
        self.bounding_size
    }

    fn set_bounds(&mut self, new_bounds: i32) -> i32 {
        if new_bounds != self.bounding_size {
            *self = CellsSingleThreaded::new();
        }
        self.bounding_size = new_bounds;
        new_bounds
    }
}