// Copyright 2025 contributors to the GeoPlegmata project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;

pub struct DggridAdapter {
    pub executable: PathBuf,
    pub workdir: PathBuf,
}

impl DggridAdapter {
    pub fn new(executable: PathBuf, workdir: PathBuf) -> Self {
        Self {
            executable,
            workdir,
        }
    }
}

impl Default for DggridAdapter {
    fn default() -> Self {
        Self {
            executable: PathBuf::from("dggrid"),
            workdir: PathBuf::from("/dev/shm"),
        }
    }
}
