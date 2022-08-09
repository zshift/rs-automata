use bevy::math::IVec3;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NeighborMethod {
    Moore,
    VonNeumann,
}

impl NeighborMethod {
    pub fn get_neighbor_iter(&self) -> &'static [IVec3] {
        match self {
            NeighborMethod::VonNeumann => &VONNEUMANN_NEIGHBORS[..],
            NeighborMethod::Moore => &MOORE_NEIGHBORS[..],
        }
    }
}

pub static VONNEUMANN_NEIGHBORS: [IVec3; 6] = [
    IVec3::from_array([1, 0, 0]),
    IVec3::from_array([-1, 0, 0]),
    IVec3::from_array([0, 1, 0]),
    IVec3::from_array([0, -1, 0]),
    IVec3::from_array([0, 0, -1]),
    IVec3::from_array([0, 0, 1]),
];

pub static MOORE_NEIGHBORS: [IVec3; 26] = [
    IVec3::from_array([-1, -1, -1]),
    IVec3::from_array([0, -1, -1]),
    IVec3::from_array([1, -1, -1]),
    IVec3::from_array([-1, 0, -1]),
    IVec3::from_array([0, 0, -1]),
    IVec3::from_array([1, 0, -1]),
    IVec3::from_array([-1, 1, -1]),
    IVec3::from_array([0, 1, -1]),
    IVec3::from_array([1, 1, -1]),
    IVec3::from_array([-1, -1, 0]),
    IVec3::from_array([0, -1, 0]),
    IVec3::from_array([1, -1, 0]),
    IVec3::from_array([-1, 0, 0]),
    IVec3::from_array([1, 0, 0]),
    IVec3::from_array([-1, 1, 0]),
    IVec3::from_array([0, 1, 0]),
    IVec3::from_array([1, 1, 0]),
    IVec3::from_array([-1, -1, 1]),
    IVec3::from_array([0, -1, 1]),
    IVec3::from_array([1, -1, 1]),
    IVec3::from_array([-1, 0, 1]),
    IVec3::from_array([0, 0, 1]),
    IVec3::from_array([1, 0, 1]),
    IVec3::from_array([-1, 1, 1]),
    IVec3::from_array([0, 1, 1]),
    IVec3::from_array([1, 1, 1]),
];
