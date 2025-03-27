mod types;
use types::{CellId, VolumeId};

/// A DGGRS provides geo-location on the Earth's surface based on a Discrete
/// Global Grid System. It translates geographic coordinates into cell
/// identifiers and vice-versa.
pub trait DGGRS {

    /// The identity of a DGGRS is ideally a URI, but possibly also a UUID. It is
    /// a short symbol telling it apart from others.
    fn identity(&self) -> String;

    /// Provides a detailed and human readable description of the system,
    /// including aspects such as datum, base polyhedra, orientation, associated map
    /// projection, aperture, sub-division method, cell indexing or any other
    /// information unequivocally identifying the DGGSRS.  
    fn description(&self) -> String;

    /// The most essential function of a DGGRS, translates a pair of geographic
    /// coordinaes into a cell identifier at a given grid refinement_levelolution.
    fn cell_id(&self, lat: i32, lon: i32, refinement_level: i16) -> CellId;
    
    /// Given a pair of geographic and a grid refinement_levelolution, returns the
    /// correfinement_levelponding cell identifier in a human readable form.
    fn cell_id_readable(&self, lat: i32, lon: i32, refinement_level: i16) -> String;

    /// Given a cell identifier, returns the geographic coordinates of its
    /// centroid.
    fn cell_centroid(&self, id: u64) -> Vec<i32>;

    /// The 3D counter part of cell_id. Takes as arguments geographic
    /// coordinates plus an altitude, returning the identifier of the
    /// correfinement_levelponding volume. A 64 bit positive integer might not provide enough
    /// precision, a different formulation is left for a later version.
    fn volume_id(&self, lat: i32, lon: i32, alt: i32, refinement_level: i16) -> VolumeId;
}


