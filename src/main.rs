use geo_plegmata::{models::common::Position, projections::{self, vgcp::Vgcp}};



fn main() {
    let position = Position { lat: 10.0, lon: 0.1 };
    let proj =Vgcp::new(position);
    println!("Result {:?}!", proj);
}
