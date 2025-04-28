use geo_plegmata::{
    models::common::Position, projections::vgcp::Vgcp, shape::icosahedron::Icosahedron,
    traits::projection::Projection,
};

fn main() {
    let position = Position {
        lat: 10.0,
        lon: 0.1,
    };
    let projection = Vgcp;

    let result = projection.forward(vec![position], &Icosahedron {});
    // let proj =Vgcp::new(position);
    println!("Result {:?}!", result);
}
