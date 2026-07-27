// Copyright 2025 contributors to the GeoPlegma project.
//
// Authored by Sunayana Ghosh (Independent Researcher, sunayanag@gmail.com)
// Licensed under the Apache License, Version 2.0 <LICENCE-APACHE-2.0 or 
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENCE-MIT 
// or http://opensource.org/licenses/MIT>, at your discretion. This file may not 
// be copied, modified or distributed except according to those terms. 

pub mod constants;
pub mod ellipsoid;
pub mod models;
pub mod projections;
pub mod utils;

// Re-export commonly used types for convenience
pub use models::vector_3d::Vector3D;