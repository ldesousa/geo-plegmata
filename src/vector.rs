// Copyright 2025 contributors to the GeoPlegmata project. 
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

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
