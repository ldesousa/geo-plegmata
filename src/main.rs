use geo_plegmata::{
    models::common::PositionGeo, projections::vgcp::Vgcp, shape::rhombic5x6::Rhombic5x6,
    traits::projection::Projection,
};

fn main() {
    let position = PositionGeo {
        lat: 38.695125,
        lon: -9.222154,
    };
    let projection = Vgcp;

    let result = projection.forward(vec![position], &Rhombic5x6 {});
    // let proj =Vgcp::new(position);
    println!("Result {:?}!", result);
}
