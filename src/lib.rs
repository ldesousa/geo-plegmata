#![doc = include_str!("../README.md")]

mod adapters;
mod ports;

pub mod models;
pub use adapters::dggrid_service::DggridService;
