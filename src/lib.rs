#![doc = include_str!("../README.md")]
pub mod adapters;
pub mod dggrs;
pub mod factory;
pub mod models;
pub mod ports;

/// This is the only re-export that is needed.
pub use factory::dggrs_factory::get;
