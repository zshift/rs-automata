mod single_threaded;
pub use single_threaded::*;

#[derive(Debug)]
struct CellState {
    value: u8,
    neighbors: u8,
}

impl CellState {
    pub fn new(value: u8, neighbors: u8) -> Self {
        Self { value, neighbors }
    }
}
