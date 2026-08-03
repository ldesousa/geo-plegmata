// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

//! One-time libhex9 lifecycle. `hex9_warp_init` builds the authalic-warp
//! interpolation state; it is idempotent and read-only afterwards, so the whole
//! library is thread-safe once initialised (the adapter is `Send + Sync`).

use std::os::raw::c_char;
use std::sync::Once;

static WARP_INIT: Once = Once::new();

/// Initialise the embedded authalic warp exactly once. Failure is non-fatal —
/// libhex9 falls back to the identity warp — and cannot occur with the blob
/// compiled into the static library, so we don't surface it.
pub fn ensure_initialised() {
    WARP_INIT.call_once(|| {
        let mut err = [0 as c_char; 256];
        // SAFETY: err is a valid, correctly-sized writable buffer.
        unsafe {
            hex9_sys::hex9_warp_init(err.as_mut_ptr(), err.len());
        }
    });
}
