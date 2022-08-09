use bevy::{math::ivec3, prelude::*};
use bevy_egui::egui;
use rand::Rng;

pub(crate) fn lerp(c1: Color, c2: Color, dt: f32) -> Color {
    let c1: Vec4 = c1.into();
    let c2: Vec4 = c2.into();
    let dt = dt.clamp(0.0, 1.0);
    ((1.0 - dt) * c1 + dt * c2).into()
}

pub(crate) fn center(bounds: i32) -> IVec3 {
    let center = bounds / 2;
    ivec3(center, center, center)
}

pub(crate) fn dist_to_center(cell_pos: IVec3, bounds: i32) -> f32 {
    let cell_pos = cell_pos - center(bounds);
    let max = bounds as f32 / 2.0;
    cell_pos.as_vec3().length() / max
}

pub(crate) fn make_some_noise_default<F: FnMut(IVec3)>(center: IVec3, f: F) {
    make_some_noise(center, 7, 12_i32.pow(3), f)
}

fn make_some_noise<F: FnMut(IVec3)>(center: IVec3, radius: i32, amount: i32, mut f: F) {
    let mut rand = rand::thread_rng();
    (0..amount).for_each(|_| {
        f(center
            + ivec3(
                rand.gen_range(-radius..=radius),
                rand.gen_range(-radius..=radius),
                rand.gen_range(-radius..=radius),
            ))
    });
}

pub(crate) fn pos_to_idx(pos: IVec3, bounds: i32) -> usize {
    let idx = pos.x + pos.y * bounds + pos.z * bounds.pow(2);
    idx as usize
}

pub(crate) fn idx_to_pos(idx: usize, bounds: i32) -> IVec3 {
    ivec3(
        idx as i32 % bounds,
        idx as i32 / bounds % bounds,
        idx as i32 / bounds.pow(2),
    )
}