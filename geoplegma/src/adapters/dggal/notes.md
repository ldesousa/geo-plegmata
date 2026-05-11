
    let args: Vec<String> = env::args().collect();
    let my_app = Application::new(&args);
    let dggal = DGGAL::new(&my_app);
    let dggrs: DGGRS = DGGRS::new(&dggal, "IVEA3H").expect("Unknown DGGRS");

    let max_depth = dggrs.getMaxDepth();
    println!("The maximum depth of this dggrs is: \n{:?}\n\n", max_depth);

    let pnt: GeoPoint = GeoPoint {
        lat: 52.3,
        lon: 12.3,
    };

    let zone = dggrs.getZoneFromWGS84Centroid(8, &pnt);
    println!("The Zone ID for the given point is: \n{:?}\n\n", zone);

    let zone_level = dggrs.getZoneLevel(zone);
    println!("The level of that Zone is: \n{:?}\n\n", zone_level);

    let zone_center = dggrs.getZoneWGS84Centroid(zone);
    println!("The center of that Zone is: \n{:?}\n\n", zone_center);

    let mut nb_types: [i32; 6] = [0; 6];
    let neighbors = dggrs.getZoneNeighbors(zone, &mut nb_types);
    println!("The neighbors of that zone: \n{:?}\n\n", neighbors);

    if zone_level + 1 > max_depth {
        println!(
            "Zone level: {:?} is already max depth {:?}",
            zone_level, max_depth
        );
    } else {
        let kids = dggrs.getSubZones(zone, 1);
        println!("The children of that zone: \n{:?}\n\n", kids);
    }

    let vertices: Vec<GeoPoint> = dggrs.getZoneWGS84Vertices(zone);
    print!("These are the vertices of the zone: \n{:?}\n\n", vertices);

    let rvertices: Vec<GeoPoint> = dggrs.getZoneRefinedWGS84Vertices(zone, 0);
    print!(
        "These are the refined vertices of the zone: \n{:?}\n\n",
        rvertices
    );

    let ll: GeoPoint = GeoPoint {
        lat: 14.5,
        lon: 14.5,
    };
    let ur: GeoPoint = GeoPoint {
        lat: 20.3,
        lon: 20.3,
    };

    let bbox = GeoExtent { ll, ur };
    println!("The extent of the bbox: \n{:?}\n\n", bbox);
    // let mut options = HashMap::<&str, &str>::new();

    // let mut exit_code: i32 = 0;

    // if parse_bbox(&options, &mut bbox) {
    //     exit_code = 1
    // }
    // println!("{:?}", bbox);
    println!("The extent of the whole world: \n{:?}\n\n", wholeWorld);

    let zones = dggrs.listZones(2, &wholeWorld);
    println!(
        "The length of the array of zone IDs for the whole world: \n{:?}\n\n",
        zones.len()
    );

    let subzones = dggrs.getSubZones(zone, 5);
    println!(
        "The length of the array of zone IDs of a parent zone: \n{:?}\n\n",
        subzones.len()
    );

    use std::time::Instant;
    let t0 = Instant::now();
    use rayon::prelude::*;
    let ga: Vec<_> = subzones
        .iter() // WARN: par_iter does not work because the underlying ecere/dggal C FFI is not threat safe.
        .map(|zone: &u64| {
            let z: u64 = *zone;
            //println!("{:?}", z);
            //let my_app2 = Application::new(&args);
            //let dggal2 = DGGAL::new(&my_app2);
            //let dggrs2: DGGRS = DGGRS::new(&dggal2, "IVEA9R").expect("Unknown DGGRS");
            dggrs.getZoneWGS84Vertices(z)
        })
        .collect();
    println!("getZoneWGS84Verticies() took {:.2?}", t0.elapsed());

