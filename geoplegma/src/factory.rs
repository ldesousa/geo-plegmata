// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::{
    dggal::grids::DggalImpl, dggrid::igeo7::Igeo7Impl, dggrid::isea3h::Isea3hImpl, h3o::h3::H3Impl,
};
use crate::api::DggrsApi;
use crate::constants::DGGRS_SPECS;
use crate::error::factory::{DggrsUidError, FactoryError};
use crate::types::{DggrsImplementation, DggrsSpec, DggrsUid};
use std::sync::Arc;

pub fn get(id: DggrsUid) -> Result<Arc<dyn DggrsApi>, FactoryError> {
    match id.spec().tool {
        DggrsImplementation::DGGRID => match id {
            DggrsUid::ISEA3HDGGRID => Ok(Arc::new(Isea3hImpl::default())),
            DggrsUid::IGEO7 => Ok(Arc::new(Igeo7Impl::default())),
            _ => Err(DggrsUidError::Unsupported { id }.into()),
        },

        DggrsImplementation::H3O => match id {
            DggrsUid::H3 => Ok(Arc::new(H3Impl::default())),
            _ => Err(DggrsUidError::Unsupported { id }.into()),
        },

        DggrsImplementation::DGGAL => match id {
            // All the DGGAL-backed IDs you support:
            DggrsUid::ISEA3HDGGAL
            | DggrsUid::IVEA3H
            | DggrsUid::ISEA9R
            | DggrsUid::IVEA9R
            | DggrsUid::RTEA3H
            | DggrsUid::RTEA9R
            | DggrsUid::IVEA7H
            | DggrsUid::IVEA7H_Z7 => Ok(Arc::new(DggalImpl::new(id))), // change DggalImpl::new to take DggrsId
            _ => Err(DggrsUidError::Unsupported { id }.into()),
        },

        DggrsImplementation::Native => Err(DggrsUidError::Unsupported { id }.into()),
    }
}

#[inline]
pub fn registry() -> &'static [DggrsSpec] {
    &DGGRS_SPECS
}
