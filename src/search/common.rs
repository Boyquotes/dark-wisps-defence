use crate::grids::common::GridCoords;

pub const CARDINAL_DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
pub const ALL_DIRECTIONS: [(i32, i32); 8] = [
    (0, 1), (1, 0), (0, -1), (-1, 0), // Cardinal directions
    (1, 1), (1, -1), (-1, 1), (-1, -1) // Diagonal directions
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub cost: usize,
    pub coords: GridCoords,
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}