use geo_plegmata::{
    models::common::Position,
    projections::vgcp::ico_vgcp::Ivgcp,
    shape::icosahedron::Icosahedron,
    traits::{polyhedron::Polyhedron, projection::Projection},
};

fn main() {
    let position = Position {
        lat: 10.0,
        lon: 0.1,
    };
    let shape = Icosahedron {};
    let projection = Ivgcp;

    let result = projection.forward(vec![position], &shape);
    // let proj =Vgcp::new(position);
    println!("Result {:?}!", result);
}
