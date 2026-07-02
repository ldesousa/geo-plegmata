// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

#[tauri::command]
async fn get_data(
    store: String,
    level: u32,
) -> Result<Vec<gp_encoding::query::VisualizationCell>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use gp_encoding::{ZarrBackend, StorageBackend};
        let backend = ZarrBackend::open(std::path::Path::new(&store)).map_err(|e| e.to_string())?;
        gp_encoding::query::export_level_as_visualization_json(&backend, level)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_data_binary(store: String, level: u32, bbox: Option<Vec<f64>>) -> Result<tauri::ipc::Response, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use gp_encoding::{StorageBackend, ZarrBackend, BoundingBox};
        let bbox = bbox.map(|b| BoundingBox::new(b[0], b[1], b[2], b[3]));
        let backend = ZarrBackend::open(std::path::Path::new(&store)).map_err(|e| e.to_string())?;
        gp_encoding::query::export_level_as_visualization_binary(&backend, level, bbox)
            .map(tauri::ipc::Response::new)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_levels(store: String) -> Result<Vec<u32>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use gp_encoding::{ZarrBackend, StorageBackend};
        let backend = ZarrBackend::open(std::path::Path::new(&store)).map_err(|e| e.to_string())?;
        Ok(backend.levels())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
struct VectorDataResponse {
    dggrs: String,
    refinement_level: u32,
    geojson: Option<serde_json::Value>,
    layers: Option<std::collections::HashMap<String, serde_json::Value>>,
}

fn resolve_cell_id_to_coord(
    cell_id_str: &str,
    dggrs: &dyn geoplegma::api::DggrsApi,
) -> Result<serde_json::Value, String> {
    use std::str::FromStr;
    use geoplegma::types::ZoneId;
    use geoplegma::api::DggrsApiConfig;

    let zone_id = ZoneId::from_str(cell_id_str).map_err(|e| e.to_string())?;
    let config = DggrsApiConfig {
        region: false,
        center: true,
        vertex_count: false,
        children: false,
        neighbors: false,
        area_sqm: false,
        densify: false,
    };
    let zones = dggrs.zone_from_id(zone_id, Some(config)).map_err(|e| e.to_string())?;
    let zone = zones.zones.into_iter().next()
        .ok_or_else(|| format!("No zone found for ID {}", cell_id_str))?;
    let center = zone.center.ok_or_else(|| format!("No center found for zone {}", cell_id_str))?;

    Ok(serde_json::json!([center.lon, center.lat]))
}

fn reconstruct_coordinates(
    coords: &mut serde_json::Value,
    geom_type: &str,
    dggrs: &dyn geoplegma::api::DggrsApi,
) -> Result<(), String> {
    match geom_type {
        "Point" => {
            if let Some(cell_id_str) = coords.as_str() {
                let coord = resolve_cell_id_to_coord(cell_id_str, dggrs)?;
                *coords = coord;
            }
        }
        "MultiPoint" | "LineString" => {
            if let Some(arr) = coords.as_array_mut() {
                for c in arr {
                    if let Some(cell_id_str) = c.as_str() {
                        *c = resolve_cell_id_to_coord(cell_id_str, dggrs)?;
                    }
                }
            }
        }
        "MultiLineString" | "Polygon" => {
            if let Some(arr2d) = coords.as_array_mut() {
                for arr1d in arr2d {
                    if let Some(arr) = arr1d.as_array_mut() {
                        for c in arr {
                            if let Some(cell_id_str) = c.as_str() {
                                *c = resolve_cell_id_to_coord(cell_id_str, dggrs)?;
                            }
                        }
                    }
                }
            }
        }
        "MultiPolygon" => {
            if let Some(arr3d) = coords.as_array_mut() {
                for arr2d in arr3d {
                    if let Some(arr2d_mut) = arr2d.as_array_mut() {
                        for arr1d in arr2d_mut {
                            if let Some(arr) = arr1d.as_array_mut() {
                                for c in arr {
                                    if let Some(cell_id_str) = c.as_str() {
                                        *c = resolve_cell_id_to_coord(cell_id_str, dggrs)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn reconstruct_geojson_in_place(
    val: &mut serde_json::Value,
    dggrs: &dyn geoplegma::api::DggrsApi,
) -> Result<(), String> {
    if let Some(obj) = val.as_object_mut() {
        let type_val = obj.get("type").and_then(|t| t.as_str().map(|s| s.to_string()));
        if let Some(type_str) = type_val {
            match type_str.as_str() {
                "Point" | "MultiPoint" | "LineString" | "MultiLineString" | "Polygon" | "MultiPolygon" => {
                    if let Some(coords) = obj.get_mut("coordinates") {
                        reconstruct_coordinates(coords, &type_str, dggrs)?;
                    }
                }
                "GeometryCollection" => {
                    if let Some(geometries) = obj.get_mut("geometries").and_then(|g| g.as_array_mut()) {
                        for geom in geometries {
                            reconstruct_geojson_in_place(geom, dggrs)?;
                        }
                    }
                }
                "Feature" => {
                    if let Some(geom) = obj.get_mut("geometry") {
                        reconstruct_geojson_in_place(geom, dggrs)?;
                    }
                }
                "FeatureCollection" => {
                    if let Some(features) = obj.get_mut("features").and_then(|f| f.as_array_mut()) {
                        for feat in features {
                            reconstruct_geojson_in_place(feat, dggrs)?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn load_vector_geojson(path: String) -> Result<VectorDataResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use std::str::FromStr;
        use geoplegma::types::DggrsUid;

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let mut value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        let dggrs_str = value
            .get("dggrs")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'dggrs' field in vector JSON".to_string())?
            .to_string();
        let refinement_level = value
            .get("refinement_level")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "Missing 'refinement_level' field in vector JSON".to_string())? as u32;

        let dggrs_uid = DggrsUid::from_str(&dggrs_str)
            .map_err(|e| format!("Unknown DGGRS: {e}"))?;
        let grid = geoplegma::get(dggrs_uid)
            .map_err(|e| format!("Failed to load DGGRS grid: {e}"))?;

        let mut geojson = None;
        let mut layers = None;

        if value.get("type").is_some() {
            reconstruct_geojson_in_place(&mut value, grid.as_ref())?;
            if let Some(obj) = value.as_object_mut() {
                obj.remove("dggrs");
                obj.remove("refinement_level");
            }
            geojson = Some(value);
        } else if let Some(layers_map) = value.get_mut("layers").and_then(|l| l.as_object_mut()) {
            let mut resolved_layers = std::collections::HashMap::new();
            for (name, val) in layers_map.iter_mut() {
                reconstruct_geojson_in_place(val, grid.as_ref())?;
                resolved_layers.insert(name.clone(), val.clone());
            }
            layers = Some(resolved_layers);
        } else {
            return Err("Vector JSON has neither a root GeoJSON type nor 'layers'".to_string());
        }

        Ok(VectorDataResponse {
            dggrs: dggrs_str.to_string(),
            refinement_level,
            geojson,
            layers,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_data,
            get_data_binary,
            get_levels,
            load_vector_geojson
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

