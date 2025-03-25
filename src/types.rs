// This is similar to the H3 Index Bit Layout but the size varais depending on the resolution
// I do not know much about DGGS so for sure I am forgetting something fundamental but this is the idea
// Also need to reason about memory aligment in case of vector of different sizes
// Bits representation:
// - bit 0: 
//          if 0 each refinement_level can be represented in two bits (aperture/refinement ratio 3, 4),
//          if 1 in three bits (aperture/refinement ratio 7). This number will be referred
//          as n_bits_hierarchy_id in the following
// - bits 1 to 3:
//          define the starting Platonic solid
// - bits 4 to (3+n_bits_hierarchy_id):
//          where n_bits_hierarchy_id = log_2(floor((size-9)/n_bits_hierarchy_id))
//          define the refinement_level/resolution (note that this definition
//          is conservative but with a LUT coulb be more efficent)
// - bits (4+n_bits_hierarchy_id) to (8+n_bits_hierarchy_id):
//          index in the Platonic solid faces
// - remaning bits:
//          each group of n_bits_hierarchy_id represent an index in the hierarchy

pub enum CellId {
    U32(u32),
    U64(u64),
    U128(u128),
}

pub enum UnitPolyhedron {
    Tetrahedron = 0,
    Cube = 1,
    Octahedron = 2,
    Dodecahedron = 3,
    Icosahedron = 4,
    Icosahedron = 5,
}

impl CellId {
    pub fn new(refinement_ratio: u8,
             initial_discrete_global_grid: UnitPolyhedron,
             refinement_level: u8, // Up to 256 refinement_levels, u128 is the bottleneck
             face_id: u8,
             hierarchy: &[u8]) -> Self {

        let n_bits_hierarchy_id = match refinement_ratio {
            3 | 4 => 2,
            7 => 3,
            _ => panic!("Valid options for refinement_ratio are 3, 4 or 7"),
        };
        let n_bits_hierarchy_id = (119 / n_bits_hierarchy_id as usize).ilog2();
        let n_bits_total = 9 + n_bits_hierarchy_id + (hierarchy.len() * n_bits_hierarchy_id as usize);
    
        let mut bits: u128 = 0;
        let mut offset = 0;
        
        // Bit 0: refinement_ratio flag
        if refinement_ratio == 7 {
            bits |= 1;
        }
        offset += 1;
    
        // Bits 1-3: initial_discrete_global_grid id
        bits |= (initial_discrete_global_grid as u128) << offset;
        offset += 3;
    
        // Bits 4 to (3+n_bits_hierarchy_id): refinement_level
        bits |= (refinement_level as u128) << offset;
        offset += n_bits_hierarchy_id as usize;
    
        // Face index (5 bits)
        bits |= (face_id as u128) << offset;
        offset += 5;
    
        // Remaining bits: hierarchal indices
        for (i, &ix) in hierarchy.iter().enumerate() {
            bits |= (ix as u128) << (offset + i * n_bits_hierarchy_id as usize);
        }
    
        // Choose type based on bit size
        if n_bits_total <= 32 {
            CellId::U32(bits as u32)
        } else if n_bits_total <= 64 {
            CellId::U64(bits as u64)
        } else if n_bits_total <= 128 {
            CellId::U128(bits)
        } else {
            panic!("The refinement_level/resolution can not be stored in 128 bits")
        }
    }
}
    

// For volumentirc ids it is assumed a radial expansion of the associated cell
// - bit 0:
//         sign, negative values are below the sea refinement_level and positive above it
// - bits 1-7:
//         refinement_level, expessed considering refinement_ratio 2.
//         The resolution define is determined by the number of refinement_level from as:
//         earth_radius / 2^refinement_level
// - bits 8 to n_bits:
//         refinement_level id.

pub enum ElevationId {
    U32(u32),
    U64(u64),
    U128(u128),
}

impl ElevationId {
    pub fn new(elevation_refinement_level: i128) -> Self {
       // ...
    }
}

pub struct VolumeId {
    pub cell: CellId,
    pub elevation: ElevationId,
}

impl VolumeId {
    pub fn new(cell: CellId, elevation_refinement_level: i128) -> Self {
        let elevation = ElevationId::new(elevation_refinement_level);
        VolumeId { cell, elevation }
    }
}



