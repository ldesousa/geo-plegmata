// Copyright 2025 contributors to the GeoPlegmata project. 
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

/// Manages the abstraction and encoding of objects implementing the Grid trait. 
pub trait GridAbstraction {
    
    /// Loads the meta-data of a Grid. Meta-data are usually encoded as the head section of a file.
    /// It must included information such as the underlying DGGRS, the cell and zone resolutions.
    fn load_metadata<T>(&self) -> &T;

    /// Loads a particular grid zone given its identifier.
    fn load_zone<T>(&self, id: u64) -> Vec<T>;

    /// Encodes the Grid meta-data. Including, but not restricted to: DGGRS, cell and zone
    /// resolutions, list of composing zones. 
    fn save_metadata(&self);

    /// Encodes a given zone in the form of a vector of values.
    fn save_zone<T>(&self, zone: Vec<T>, id: u64);
}

/// Manages the abstraction and encoding of objects implementing the Vector trait. 
pub trait VectorAbstraction {
    
    /// Loads the meta-data block of a Vector. Identifies the underlying DGGRS, geometry type,
    /// number of features, attributes, etc. 
    fn load_metadata<T>(&self) -> &T;

    /// Loads a particular geometry from a given feature id.
    fn load_geometry<T>(&self, id: u64) -> Vec<T>;

    /// Loads a particular attributes record from a given feature id.
    fn load_attributes<T>(&self, id: u64) -> Vec<T>;

    /// Encodes the Vector meta-data. Including, but not restricted to: DGGRS, list of attributes,
    /// list of feature identifiers.  
    fn save_metadata(&self);

    /// Encodes a given zone in the form of a vector of values.
    fn save_feature<T, R>(&self, geom: Vec<T>, record: &R);

}
