// Copyright 2025 contributors to the GeoPlegmata project. 
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

/// A Grid is map from cell identifiers to values. These values may represent
/// any geo-spatial variable of interest. Grids are organised into zones, large
/// blocks of contiguous cells (with associated values). A zone is typically the
/// unit of grid processing.
///
/// To do:
/// - Cell iterator
/// - Zone iterator
pub trait Grid {

    /// Identifies the DGGRS on which the Grid is based.
    fn dggrs(&self) -> String;

    /// Identifies the data type of the values stored by the Grid.
    fn type(&self) -> String;

    /// Identifies the resolution of the Grid respective to the underlying DGGRS. Determines the
    /// size of the grid cell.
    fn resolution(&self) -> String;

    /// Identifies the resolution of the zone(s) making the up the grid. The
    /// zone resolution must be smaller than that of the cell.
    fn zone_resolution(&self) -> String;

    /// Returns a vector with the identifiers of all zones composing the grid.
    /// Zones are typically identified by cell identifiers at the zone
    /// resolution. But other schemes may be possible.
    fn zone_ids(&self) -> Vec<u64>

    /// Returns the value associated with a particular grid cell.
    fn get<T>(&self, id: u64) -> &T;

    /// Sets the value associated with a particular grid cell.
    fn set<T>(&self, id: u64, &T);

    /// Returns a vector with all the values of a particular zone. The vector is
    /// principle ordered according to space filling curve method.
    fn get_zone<T>(&self, id: u64) -> Vec<T>;
}
