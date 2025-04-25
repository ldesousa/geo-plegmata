pub trait Polyhedron {
    fn new(&mut self) ;
    fn planar_vertexes(&mut self) ;
    fn unit_vectors(&mut self);
    fn triangle_arc_lengths(&mut self);
}
