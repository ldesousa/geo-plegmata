use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::path::Path;
use gp_encoding::{StorageBackend, ZarrBackend};

#[pyclass]
pub struct Store {
    backend: ZarrBackend,
}

#[pymethods]
impl Store {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let backend = ZarrBackend::open(Path::new(&path))
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open store: {}", e)))?;
        Ok(Store { backend })
    }

    fn levels(&self) -> PyResult<Vec<u32>> {
        Ok(self.backend.levels())
    }

    fn export_level(&self, level: u32) -> PyResult<String> {
        let cells = gp_encoding::query::export_level_as_visualization_json(&self.backend, level)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to export level: {}", e)))?;
        
        let json_str = serde_json::to_string(&cells)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize to JSON: {}", e)))?;
        
        Ok(json_str)
    }
}

/// A Python module implemented in Rust for GeoPlegma.
#[pymodule]
fn geoplegma_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Store>()?;
    Ok(())
}
