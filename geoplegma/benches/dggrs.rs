use api::get;
use api::models::common::{DggrsUid, RefinementLevel, RelativeDepth};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use geo::{Point, Rect};

fn bench_dggrs(c: &mut Criterion) {
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

    let points = vec![Point::new(19.96, 5.34)];
    let bbox = Rect::new(Point::new(-10.0, -10.0), Point::new(10.0, 10.0));

    let options = api::config {
        region: true,
        children: false,
        center: false,
        neighbors: false,
        densify: false,
        area_sqm: false,
        ..Default::default()
    };

    let mut group = c.benchmark_group("dggrs");

    for did in dt {
        let d = get(did).expect("DGGRS not available");

        for &p in &points {
            for lrf in 1..=4i32 {
                let rf = RefinementLevel::new(lrf).expect("RefinementLevel expected");

                // --- zone_from_point ---
                group.bench_with_input(
                    BenchmarkId::new(
                        "zone_from_point",
                        format!("{}/{}/rf{}", did.spec().name, lrf),
                    ),
                    &(),
                    |b, _| {
                        b.iter(|| {
                            let r = d.zone_from_point(rf, p, Some(options.clone())).unwrap();
                            black_box(r);
                        })
                    },
                );

                // Setup: get a zone id once (not part of the measurements below)
                let setup = d.zone_from_point(rf, p, Some(options.clone())).unwrap();
                let zone_id = setup.zones[0].id.clone();

                // --- zones_from_parent ---
                let max_rd = d.max_relative_depth().unwrap().get();
                for lrd in 1..=max_rd {
                    let rd = RelativeDepth::new(lrd).unwrap();
                    group.bench_with_input(
                        BenchmarkId::new(
                            "zones_from_parent",
                            format!("{}/rf{}/rd{}", did.spec().name, lrf, lrd),
                        ),
                        &(),
                        |b, _| {
                            b.iter(|| {
                                let r = d
                                    .zones_from_parent(rd, zone_id.clone(), Some(options.clone()))
                                    .unwrap();
                                black_box(r);
                            })
                        },
                    );
                }

                // --- zone_from_id ---
                group.bench_with_input(
                    BenchmarkId::new("zone_from_id", format!("{}/rf{}", did.spec().name, lrf)),
                    &(),
                    |b, _| {
                        b.iter(|| {
                            let r = d
                                .zone_from_id(zone_id.clone(), Some(options.clone()))
                                .unwrap();
                            black_box(r);
                        })
                    },
                );

                // --- zones_from_bbox (local) ---
                group.bench_with_input(
                    BenchmarkId::new("zones_from_bbox", format!("{}/rf{}", did.spec().name, lrf)),
                    &(),
                    |b, _| {
                        b.iter(|| {
                            let r = d
                                .zones_from_bbox(rf, Some(bbox), Some(options.clone()))
                                .unwrap();
                            black_box(r);
                        })
                    },
                );

                // --- zones_from_bbox (global) ---
                let global_rf = RefinementLevel::new(1).unwrap();
                group.bench_with_input(
                    BenchmarkId::new("zones_from_bbox_global", did.spec().name),
                    &(),
                    |b, _| {
                        b.iter(|| {
                            let r = d
                                .zones_from_bbox(global_rf, None, Some(options.clone()))
                                .unwrap();
                            black_box(r);
                        })
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_dggrs);
criterion_main!(benches);
