use geo_plegmata::{
    models::common::PositionGeo, projections::vgcp::Vgcp, shape::icosahedron::Icosahedron,
    traits::projection::Projection,
};

fn main() {
    let position = PositionGeo {
        lat: 38.695125,
        lon: -9.222154
    };
    let projection = Vgcp;

    let result = projection.forward(vec![position], &Icosahedron {});
    // let proj =Vgcp::new(position);
    println!("Result {:?}!", result);
}
