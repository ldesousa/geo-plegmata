
/// A collection of geometric features with associated information. It is essence composed by two
/// information elements: a vector of geometries and an attribute table. The later contains a
/// record for each geometry.
///
/// To do:
/// - almost everything
pub trait Vector {

    /// Identifies the DGGRS on which the Grid is based.
    fn dggrs(&self) -> String;

}
