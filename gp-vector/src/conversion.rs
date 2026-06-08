// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::ConversionError;
use geoplegma::api::DggrsApi;
use geoplegma::types::{DggrsUid, RefinementLevel};
use serde_json::Value;

fn convert_coord_array(
    val: &Value,
    dggrs: &dyn DggrsApi,
    refinement_level: RefinementLevel,
) -> Result<String, ConversionError> {
    let arr = val.as_array().ok_or(ConversionError::InvalidCoordinateFormat)?;
    if arr.len() < 2 {
        return Err(ConversionError::InvalidCoordinateLength(arr.len()));
    }
    let lon = arr[0].as_f64().ok_or(ConversionError::InvalidCoordinateFormat)?;
    let lat = arr[1].as_f64().ok_or(ConversionError::InvalidCoordinateFormat)?;

    let point = geoplegma::types::Point::new(lat, lon);
    let config = geoplegma::api::DggrsApiConfig {
        region: false,
        center: false,
        vertex_count: false,
        children: false,
        neighbors: false,
        area_sqm: false,
        densify: false,
    };

    let zones = dggrs.zone_from_point(refinement_level, point, Some(config))?;
    let zone_id = zones
        .zones
        .into_iter()
        .next()
        .ok_or(ConversionError::NoZoneFound(point))?
        .id;

    Ok(zone_id.to_string())
}

fn convert_coordinates(
    coords: &mut Value,
    geom_type: &str,
    dggrs: &dyn DggrsApi,
    refinement_level: RefinementLevel,
) -> Result<(), ConversionError> {
    match geom_type {
        "Point" => {
            let cell_id = convert_coord_array(coords, dggrs, refinement_level)?;
            *coords = Value::String(cell_id);
        }
        "MultiPoint" | "LineString" => {
            if let Some(arr) = coords.as_array_mut() {
                let mut new_arr = Vec::with_capacity(arr.len());
                for c in arr {
                    let cell_id = convert_coord_array(c, dggrs, refinement_level)?;
                    new_arr.push(Value::String(cell_id));
                }
                *coords = Value::Array(new_arr);
            }
        }
        "MultiLineString" | "Polygon" => {
            if let Some(arr2d) = coords.as_array_mut() {
                for arr1d in arr2d {
                    if let Some(arr) = arr1d.as_array_mut() {
                        let mut new_arr = Vec::with_capacity(arr.len());
                        for c in arr {
                            let cell_id = convert_coord_array(c, dggrs, refinement_level)?;
                            new_arr.push(Value::String(cell_id));
                        }
                        *arr1d = Value::Array(new_arr);
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
                                let mut new_arr = Vec::with_capacity(arr.len());
                                for c in arr {
                                    let cell_id = convert_coord_array(c, dggrs, refinement_level)?;
                                    new_arr.push(Value::String(cell_id));
                                }
                                *arr1d = Value::Array(new_arr);
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

pub fn convert_geojson_in_place(
    val: &mut Value,
    dggrs: &dyn DggrsApi,
    refinement_level: RefinementLevel,
) -> Result<(), ConversionError> {
    if let Some(obj) = val.as_object_mut() {
        let type_val = obj.get("type").and_then(|t| t.as_str().map(|s| s.to_string()));
        if let Some(type_str) = type_val {
            match type_str.as_str() {
                "Point" | "MultiPoint" | "LineString" | "MultiLineString" | "Polygon" | "MultiPolygon" => {
                    if let Some(coords) = obj.get_mut("coordinates") {
                        convert_coordinates(coords, &type_str, dggrs, refinement_level)?;
                    }
                }
                "GeometryCollection" => {
                    if let Some(geometries) = obj.get_mut("geometries").and_then(|g| g.as_array_mut()) {
                        for geom in geometries {
                            convert_geojson_in_place(geom, dggrs, refinement_level)?;
                        }
                    }
                }
                "Feature" => {
                    if let Some(geom) = obj.get_mut("geometry") {
                        convert_geojson_in_place(geom, dggrs, refinement_level)?;
                    }
                }
                "FeatureCollection" => {
                    if let Some(features) = obj.get_mut("features").and_then(|f| f.as_array_mut()) {
                        for feat in features {
                            convert_geojson_in_place(feat, dggrs, refinement_level)?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn convert_to_document(
    mut geojson: Value,
    dggrs_uid: DggrsUid,
    refinement_level: RefinementLevel,
) -> Result<Value, ConversionError> {
    let dggrs = geoplegma::get(dggrs_uid)?;
    convert_geojson_in_place(&mut geojson, dggrs.as_ref(), refinement_level)?;

    if let Some(obj) = geojson.as_object_mut() {
        let mut new_map = serde_json::Map::new();
        new_map.insert("dggrs".to_string(), Value::String(dggrs_uid.to_string()));
        new_map.insert("refinement_level".to_string(), Value::Number(refinement_level.get().into()));

        let old_map = std::mem::take(obj);
        for (k, v) in old_map {
            new_map.insert(k, v);
        }

        Ok(Value::Object(new_map))
    } else {
        let mut doc_map = serde_json::Map::new();
        doc_map.insert("dggrs".to_string(), Value::String(dggrs_uid.to_string()));
        doc_map.insert("refinement_level".to_string(), Value::Number(refinement_level.get().into()));
        doc_map.insert("geojson".to_string(), geojson);
        Ok(Value::Object(doc_map))
    }
}
