
use geo_plegmata::types::{CellId, UnitPolyhedron};

#[test]
fn test_cell_id_small() {
    let refinement_ratio = 4_u8;
    let initial_discrete_global_grid = UnitPolyhedron::Icosahedron;
    let face_id = 5_u8;
    let hierarchy = [2, 3];
    let cell_id = CellId::new(refinement_ratio,
                         initial_discrete_global_grid,
                         face_id,
                         &hierarchy);

    let bits = cell_id.bits();
    let total_bits = cell_id.bit_length();

    println!("CellId = {} ({} bits):", bits, total_bits);
    for i in (0..total_bits).rev() {
        let bit = (bits >> i) & 1;
        print!("{}", bit);
        if i % 8 == 0 { print!(" "); } // optional: group by byte
    }
    println!();

    assert!(bits == 231976, "Cell ID not matching the reference");
    match cell_id {
        CellId::U32(_) => {}
        _ => panic!("Expected U32 for small hierarchy"),
    }
}

