// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geoplegma::types::{DggrsUid, RefinementLevel};
use gp_vector::convert_to_document;

#[test]
fn test_point_conversion() {
    let geojson = serde_json::json!({
        "type": "Point",
        "coordinates": [12.4924, 41.8902]
    });

    let refinement_level = RefinementLevel::new(7).unwrap();
    let doc = convert_to_document(geojson, DggrsUid::H3, refinement_level).unwrap();

    println!("Converted document: {}", doc);
    assert_eq!(doc["dggrs"], "H3");
    assert_eq!(doc["refinement_level"], 7);
    assert_eq!(doc["type"], "Point");
    assert!(doc["coordinates"].is_string());
    assert!(!doc["coordinates"].as_str().unwrap().is_empty());
}

#[test]
fn test_linestring_conversion() {
    let geojson = serde_json::json!({
        "type": "LineString",
        "coordinates": [
            [12.4924, 41.8902],
            [12.4934, 41.8912]
        ]
    });

    let refinement_level = RefinementLevel::new(7).unwrap();
    let doc = convert_to_document(geojson, DggrsUid::H3, refinement_level).unwrap();

    println!("Converted document: {}", doc);

    assert_eq!(doc["type"], "LineString");
    let arr = doc["coordinates"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0].is_string());
    assert!(arr[1].is_string());
}

#[test]
fn test_polygon_conversion() {
    let geojson = serde_json::json!({
        "type": "Polygon",
        "coordinates": [
            [
                [12.49, 41.89],
                [12.50, 41.89],
                [12.50, 41.90],
                [12.49, 41.90],
                [12.49, 41.89]
            ]
        ]
    });

    let refinement_level = RefinementLevel::new(6).unwrap();
    let doc = convert_to_document(geojson, DggrsUid::H3, refinement_level).unwrap();

    println!("Converted document: {}", doc);

    assert_eq!(doc["type"], "Polygon");
    let rings = doc["coordinates"].as_array().unwrap();
    assert_eq!(rings.len(), 1);
    let coords = rings[0].as_array().unwrap();
    assert_eq!(coords.len(), 5);
    for c in coords {
        assert!(c.is_string());
    }
}

#[test]
fn test_feature_collection_conversion() {
    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [12.4924, 41.8902]
                },
                "properties": {
                    "name": "Colosseum"
                }
            }
        ]
    });

    let refinement_level = RefinementLevel::new(7).unwrap();
    let doc = convert_to_document(geojson, DggrsUid::H3, refinement_level).unwrap();

    println!("Converted document: {}", doc);
    assert_eq!(doc["type"], "FeatureCollection");
    let features = doc["features"].as_array().unwrap();
    assert_eq!(features.len(), 1);
    let feat = &features[0];
    assert_eq!(feat["type"], "Feature");
    assert_eq!(feat["properties"]["name"], "Colosseum");

    let geom = &feat["geometry"];
    assert_eq!(geom["type"], "Point");
    assert!(geom["coordinates"].is_string());
}

#[test]
fn test_larger_feature_collection_conversion() {
    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {"type": "Point", "coordinates": [102.0, 0.5]},
                "properties": {"prop0": "value0"}
            },
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [
                    [102.0, 0.0], [103.0, 1.0], [104.0, 0.0], [105.0, 1.0]
                    ]
                },
                "properties": {
                    "prop0": "value0",
                    "prop1": 0.0
                }
            },
            {
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [
                    [ [100.0, 0.0], [101.0, 0.0], [101.0, 1.0],
                        [100.0, 1.0], [100.0, 0.0] ]
                    ]
            },
            "properties": {
                "prop0": "value0",
                "prop1": {"this": "that"}
            }
            }
        ]
      }
    );


    let refinement_level = RefinementLevel::new(7).unwrap();
    let doc = convert_to_document(geojson, DggrsUid::H3, refinement_level).unwrap();

    println!("Converted document: {}", doc);
    assert_eq!(doc["type"], "FeatureCollection");
    let features = doc["features"].as_array().unwrap();
    assert_eq!(features.len(), 3);
}
