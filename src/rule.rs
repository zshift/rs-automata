use std::ops::RangeInclusive;

use crate::neighbors::NeighborMethod;

#[derive(Clone, PartialEq)]
pub struct Rule {
    pub survival_rule: Value,
    pub birth_rule: Value,
    pub states: u8,
    pub neighbor_method: NeighborMethod,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Value ([bool; 27]);
impl Value {
    pub fn new(indices: &[u8]) -> Self {
        let mut result  = Value([false; 27]);
        for index in indices {
            result.0[*index as usize] = true;
        }
        result
    }

    pub fn from_range(indices: RangeInclusive<i32>) -> Value {
        let mut result = Value([false; 27]);
        for idx in indices {
            result.0[idx as usize] = true;
        }
        result
    }
}