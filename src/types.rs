// This is similar to the H3 Index Bit Layout but the size varais depending on the resolution
// I do not know much about DGGS so for sure I am forgetting something fundamental but this is the idea
// Also need to reason about memory aligment in case of vector of different sizes
// Bits representation:
// - bit 0: 
//          if 0 each level can be represented in two bits (aperture 3, 4),
//          if 1 in three bits (aperture 7). This number will be referred
//          as n_bh in the following
// - bits 1 to 3:
//          define the starting Platonic solid
// - bits 4 to (3+n_bl):
//          where n_bl = log_2(floor((size-9)/n_bh))
//          define the level/resolution (note that this definition
//          is conservative but with a LUT coulb be more efficent)
// - bits (4+n_bl) to (8+n_bl):
//          index in the Platonic solid faces
// - remaning bits:
//          each group of n_bh represent an index in the hierarchy

pub enum CellId {
    U32(u32),
    U64(u64),
    U128(u128),
}

pub enum PlatonicSolid {
    Tetrahedron = 0,
    Cube = 1,
    Octahedron = 2,
    Dodecahedron = 3,
    Icosahedron = 4
}

impl CellId {
  pub fn new(aperture: u8,
             solid: PaltonicSolid,
             level: u8, // Up to 256 levels, u128 is the bottleneck
             face_id: u8,
             h_ids: &[u8]) -> Self {
    let n_bh = match aperture {
      3 | 4 => 2,
      7 => 3,
      _ => panic!("Valid options for aperture are 3, 4 or 7"),
    }:
    let n_bl = (119 / n_bh as usize).ilog2();
    let n_bits = 9 + n_bl + (h_ids.len() * n_bh as usize);

    let mut bits: u128 = 0;
    let mut offset = 0;
    
    // Bit 0: aperture flag
    if aperture == 7 {
      bits |= 1;
    }
    offset += 1;

    // Bits 1-3: Platonic solid
    bits |= (solid as u128) << offset;
    offset += 3;

    // Bits 4 to (3+n_bl): level
    bits |= (level as u128) << offset;
    offset += n_bl as usize;

    // Face index (5 bits)
    bits |= (face_id as u128) << offset;
    offset += 5;

    // Remaining bits: hierarchal indices
    for (i, &ix) in h_ids.iter().enumerate() {
        bits |= (ix as u128) << (offset + i * n_bh as usize);
    }

    // Choose type based on bit size
    if n_bits <= 32 {
      CellId::U32(bits as u32)
    } else if n_bits <= 64 {
      CellId::U64(bits as u64)
    } else if n_bits <= 128 {
      CellId::U128(bits)
    } else {
      panic!("The level/resolution can not be stored in 128 bits")
    }
  }
}
    
  
