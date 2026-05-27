// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geoplegma::api::DggrsApiConfig;

pub(crate) const CONFIG: DggrsApiConfig = DggrsApiConfig {
    region: true,
    children: false,
    center: false,
    neighbors: false,
    densify: false,
    area_sqm: false,
    vertex_count: false,
};

pub(crate) const ID_ONLY_CONFIG: DggrsApiConfig = DggrsApiConfig {
    region: false,
    children: false,
    center: false,
    neighbors: false,
    densify: false,
    area_sqm: false,
    vertex_count: false,
};
pub(crate) const CENTER_CONFIG: DggrsApiConfig = DggrsApiConfig {
    region: false,
    children: false,
    center: true,
    neighbors: false,
    densify: false,
    area_sqm: false,
    vertex_count: false,
};
