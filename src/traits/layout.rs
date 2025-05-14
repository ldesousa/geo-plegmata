use crate::models::common::Position2D;


pub trait Layout {
    fn face_center(&self, vertices: [(u8, u8); 3]) -> Position2D;
    fn grid_size(&self) -> (usize, usize);
    fn vertices(&self) -> Vec<[(u8, u8); 3]>;
}
