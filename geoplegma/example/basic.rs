// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use geoplegma::error;
use geoplegma::types::{DggrsUid, RefinementLevel, RelativeDepth};
use geoplegma::{get, registry};
use geoplegma::api::{BoundingBox, Point};
use std::time::Instant;

/// This is just an example and basic testing function if there is output or not
pub fn main() -> Result<(), error::DggrsError> {
    println!("{:?}", registry());

    let dt = vec![
        DggrsUid::ISEA3HDGGRID,
        DggrsUid::IGEO7,
        DggrsUid::H3,
        DggrsUid::ISEA3HDGGAL,
        DggrsUid::IVEA3H,
        DggrsUid::IVEA9R,
        DggrsUid::IVEA3H,
        DggrsUid::RTEA9R,
        DggrsUid::RTEA3H,
        DggrsUid::IVEA7H,
        DggrsUid::IVEA7H_Z7,
    ];

    let points = vec![
        Point::new(5.34, 19.96),
        //        Point::new(52.98, 9.06),
        //      Point::new(-15.28, -29.11),
    ];

    let bbox = BoundingBox::new(-10.0, -10.0, 10.0, 10.0);

    let mut options = geoplegma::config {
        region: true,
        children: false,
        center: false,
        neighbors: false,
        densify: false,
        area_sqm: false,
        ..Default::default()
    };
    let mut now = Instant::now();
    let mut elapsed = now.elapsed();
    let print_first = false;
    for did in &dt {
        println!(
            "\n\n=== DGGRS: {} TOOL: {} ===",
            &did.spec().name,
            &did.spec().tool,
        );
        let d = get(*did).unwrap();

        for p in &points {
            println!("=== POINT: {:?} ===", &p);

            for lrf in 1..=4 {
                let rf = RefinementLevel::new(lrf)?;

                println!("=== Refinment Level: {:?} ===", &rf);
                now = Instant::now();
                let r = d.zone_from_point(rf, *p, Some(options))?;
                elapsed = now.elapsed();
                println!(
                    "zone_from_point generated {} zones in {:.2?}",
                    r.zones.len(),
                    elapsed
                );
                if print_first {
                    if let Some(first) = r.zones.get(0) {
                        println!("\nhere is the first entry\n{first:?}");
                    }
                }

                let zone = &r.zones[0].id;
                for lrd in 1..=d.max_relative_depth()?.get() {
                    let relative_depth = RelativeDepth::new(lrd)?;
                    now = Instant::now();
                    let r = d.zones_from_parent(relative_depth, zone.clone(), Some(options))?;
                    elapsed = now.elapsed();
                    println!(
                        "zones_from_parent for relative depth {:>2} generated {:>10} zones in {:>12.2?}",
                        relative_depth.get(),
                        r.zones.len(),
                        elapsed
                    );
                    if print_first {
                        if let Some(first) = r.zones.get(0) {
                            println!("\nhere is the first entry\n{first:?}");
                        }
                    }
                }

                now = Instant::now();
                let r = d.zone_from_id(zone.clone(), Some(options))?;
                elapsed = now.elapsed();
                println!(
                    "zone_from_id generated {} zones in {:.2?}",
                    r.zones.len(),
                    elapsed
                );
                if print_first {
                    if let Some(first) = r.zones.get(0) {
                        println!("\nhere is the first entry\n{first:?}");
                    }
                }

                now = Instant::now();
                let r = d.zones_from_bbox(rf, Some(bbox), Some(options))?;
                elapsed = now.elapsed();
                println!(
                    "zone_from_bbox generated {} zones in {:.2?}",
                    r.zones.len(),
                    elapsed
                );
                if print_first {
                    if let Some(first) = r.zones.get(0) {
                        println!("\nhere is the first entry\n{first:?}");
                    }
                }

                let global_rf = RefinementLevel::new(1)?;
                now = Instant::now();
                let r = d.zones_from_bbox(global_rf, None, Some(options))?;
                elapsed = now.elapsed();
                println!(
                    "zone_from_bbox (global) generated {} zones in {:.2?}",
                    r.zones.len(),
                    elapsed
                );
                if print_first {
                    if let Some(first) = r.zones.get(0) {
                        println!("here is the first entry\n{first:?}");
                    }
                }
            }
        }
    }
    Ok(())
}
