// This is similar to the H3 Index Bit Layout but the size varais depending on the resolution
// I do not know much about DGGS so for sure I am forgetting something fundamental but this is the idea

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
