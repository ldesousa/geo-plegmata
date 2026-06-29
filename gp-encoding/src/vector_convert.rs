// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::EncodingError;
use gdal::vector::{FieldValue, LayerAccess};
use gdal::Dataset;
use geoplegma::api::DggrsApi;
use geoplegma::types::{DggrsUid, RefinementLevel};
use geo_types::Geometry;
use serde_json::{json, Value};
use std::path::Path;

fn field_value_to_json(val: FieldValue) -> Value {
    match val {
        FieldValue::IntegerValue(i) => Value::Number(i.into()),
        FieldValue::IntegerListValue(list) => Value::Array(list.into_iter().map(|i| Value::Number(i.into())).collect()),
        FieldValue::Integer64Value(i) => Value::Number(i.into()),
        FieldValue::Integer64ListValue(list) => Value::Array(list.into_iter().map(|i| Value::Number(i.into())).collect()),
        FieldValue::StringValue(s) => Value::String(s),
        FieldValue::StringListValue(list) => Value::Array(list.into_iter().map(Value::String).collect()),
        FieldValue::RealValue(f) => Value::from(f),
        FieldValue::RealListValue(list) => Value::Array(list.into_iter().map(Value::from).collect()),
        FieldValue::DateValue(d) => Value::String(d.to_string()),
        FieldValue::DateTimeValue(dt) => Value::String(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
    }
}

fn geo_to_geojson_value(geom: &Geometry<f64>) -> Value {
    match geom {
        Geometry::Point(p) => json!({
            "type": "Point",
            "coordinates": [p.x(), p.y()]
        }),
        Geometry::LineString(ls) => json!({
            "type": "LineString",
            "coordinates": ls.coords().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
        }),
        Geometry::Polygon(poly) => {
            let mut coords = vec![
                poly.exterior().coords().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
            ];
            for ring in poly.interiors() {
                coords.push(ring.coords().map(|c| vec![c.x, c.y]).collect::<Vec<_>>());
            }
            json!({
                "type": "Polygon",
                "coordinates": coords
            })
        },
        Geometry::MultiPoint(mp) => json!({
            "type": "MultiPoint",
            "coordinates": mp.iter().map(|p| vec![p.x(), p.y()]).collect::<Vec<_>>()
        }),
        Geometry::MultiLineString(mls) => json!({
            "type": "MultiLineString",
            "coordinates": mls.iter().map(|ls| ls.coords().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()).collect::<Vec<_>>()
        }),
        Geometry::MultiPolygon(mpoly) => {
            let coords = mpoly.iter().map(|poly| {
                let mut ring_coords = vec![
                    poly.exterior().coords().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
                ];
                for ring in poly.interiors() {
                    ring_coords.push(ring.coords().map(|c| vec![c.x, c.y]).collect::<Vec<_>>());
                }
                ring_coords
            }).collect::<Vec<_>>();
            json!({
                "type": "MultiPolygon",
                "coordinates": coords
            })
        },
        Geometry::GeometryCollection(gc) => json!({
            "type": "GeometryCollection",
            "geometries": gc.iter().map(geo_to_geojson_value).collect::<Vec<_>>()
        }),
        _ => Value::Null,
    }
}

fn convert_coord_array(
    val: &Value,
    dggrs: &dyn DggrsApi,
    refinement_level: RefinementLevel,
) -> Result<String, EncodingError> {
    let arr = val.as_array().ok_or(EncodingError::InvalidCoordinateFormat)?;
    if arr.len() < 2 {
        return Err(EncodingError::InvalidCoordinateLength(arr.len()));
    }
    let lon = arr[0].as_f64().ok_or(EncodingError::InvalidCoordinateFormat)?;
    let lat = arr[1].as_f64().ok_or(EncodingError::InvalidCoordinateFormat)?;

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
        .ok_or(EncodingError::NoZoneFound(point))?
        .id;

    Ok(zone_id.to_string())
}

fn convert_coordinates(
    coords: &mut Value,
    geom_type: &str,
    dggrs: &dyn DggrsApi,
    refinement_level: RefinementLevel,
) -> Result<(), EncodingError> {
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
) -> Result<(), EncodingError> {
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

pub fn convert_vector_file_to_json(
    input_path: &Path,
    output_path: &Path,
    dggrs_uid: DggrsUid,
    refinement_level: RefinementLevel,
) -> Result<(), EncodingError> {
    let dataset = Dataset::open(input_path)?;
    let layer_count = dataset.layer_count();
    let grid = geoplegma::get(dggrs_uid)?;

    let mut doc_map = serde_json::Map::new();
    doc_map.insert("dggrs".to_string(), Value::String(dggrs_uid.to_string()));
    doc_map.insert("refinement_level".to_string(), Value::Number(refinement_level.get().into()));

    if layer_count == 1 {
        let mut layer = dataset.layer(0)?;
        let defn = layer.defn();
        let fields_schema: Vec<(usize, String)> = defn
            .fields()
            .enumerate()
            .map(|(i, f)| (i, f.name()))
            .collect();

        let mut features_arr = Vec::new();
        for feature in layer.features() {
            let mut properties = serde_json::Map::new();
            for (i, name) in &fields_schema {
                if let Some(val) = feature.field(*i)? {
                    properties.insert(name.clone(), field_value_to_json(val));
                } else {
                    properties.insert(name.clone(), Value::Null);
                }
            }

            let mut geom_val = Value::Null;
            if let Some(gdal_geom) = feature.geometry() {
                let geo_geom = gdal_geom.to_geo()?;
                geom_val = geo_to_geojson_value(&geo_geom);
                convert_geojson_in_place(&mut geom_val, grid.as_ref(), refinement_level)?;
            }

            features_arr.push(json!({
                "type": "Feature",
                "geometry": geom_val,
                "properties": properties
            }));
        }

        doc_map.insert("type".to_string(), Value::String("FeatureCollection".to_string()));
        doc_map.insert("features".to_string(), Value::Array(features_arr));
    } else {
        let mut layers_map = serde_json::Map::new();
        for idx in 0..layer_count {
            let mut layer = dataset.layer(idx)?;
            let layer_name = layer.name();
            let defn = layer.defn();
            let fields_schema: Vec<(usize, String)> = defn
                .fields()
                .enumerate()
                .map(|(i, f)| (i, f.name()))
                .collect();

            let mut features_arr = Vec::new();
            for feature in layer.features() {
                let mut properties = serde_json::Map::new();
                for (i, name) in &fields_schema {
                    if let Some(val) = feature.field(*i)? {
                        properties.insert(name.clone(), field_value_to_json(val));
                    } else {
                        properties.insert(name.clone(), Value::Null);
                    }
                }

                let mut geom_val = Value::Null;
                if let Some(gdal_geom) = feature.geometry() {
                    let geo_geom = gdal_geom.to_geo()?;
                    geom_val = geo_to_geojson_value(&geo_geom);
                    convert_geojson_in_place(&mut geom_val, grid.as_ref(), refinement_level)?;
                }

                features_arr.push(json!({
                    "type": "Feature",
                    "geometry": geom_val,
                    "properties": properties
                }));
            }

            let layer_collection = json!({
                "type": "FeatureCollection",
                "features": features_arr
            });
            layers_map.insert(layer_name, layer_collection);
        }
        doc_map.insert("layers".to_string(), Value::Object(layers_map));
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(output_path)?;
    serde_json::to_writer_pretty(file, &Value::Object(doc_map))?;

    Ok(())
}
