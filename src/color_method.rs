use bevy::prelude::*;
use crate::utils;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorMethod {
    Single,
    StateLerp,
    DistToCenter,
    Neighbor,
}

impl ColorMethod {
    pub fn color(
        &self,
        c1: Color,
        c2: Color,
        states: u8,
        state: u8,
        neigbors: u8,
        dist_to_center: f32,
    ) -> Color {
        match self {
            ColorMethod::Single => c1,
            ColorMethod::StateLerp => {
                let dt = state as f32 / states as f32;
                utils::lerp(c1, c2, dt)
            }
            ColorMethod::DistToCenter => {
                utils::lerp(c1, c2, dist_to_center)
            }
            ColorMethod::Neighbor => {
                let dt = neigbors as f32 / 26.0;
                utils::lerp(c1, c2, dt)
            }
        }
    }
}
