# DGGRS Library
This library provides an interface to DGGRID (and potentinally other tools) to genereate cells. The output of the three public functions is a `CellsGEO` struct with the cell ID and an vector of coordinates that describes the cell polygon using the [geo](https://github.com/georust/geo) primitive [Polygon](https://docs.rs/geo/latest/geo/geometry/struct.Polygon.html).

## Requirments

Make sure DGGRID is compiled and available on your system. Remember the path where the `dggrid` executable is, or add `dggrid` to your `$PATH`.

## Usage Example

Create a new crate with `cargo new` and add this dependency in your `cargo.toml`. I expect to publish this to crates.io in the future, which will simplify this with `cargo add dggrs`.
````
[dependencies]
dggrs = {version = "0.1.0", git = git@gitlab.com/geoinsight/dggrs.git}
````

In your `main.rs` add the following code. In this example the DGGRID generator service is instantiated using the path to the DGGRID executable `dggrid` and a path to the work directory `/dev/shm`. 

````
use dggrs;
use dggrs::models::dggrid::CellID;
use geo::geometry::Point;

fn main() {
    let generator = dggrs::DggridService::default();

    let configs = vec![
        (
            "ISEA3H",
            CellID::new("x000000000000000").expect("invalid ISEA3H ID"),
        ),
        (
            "IGEO7",
            CellID::new("099fffffffffffff").expect("invalid IGEO7 ID"),
        ),
    ];

    let bbox: Option<Vec<Vec<f64>>> = Some(vec![
        vec![-77.0, 39.0], // lower left
        vec![-76.0, 40.0], // upper right
    ]);

    let pnt = Point::new(10.9, 4.9);
    for (dggs, cell_id) in configs {
        println!("=== DGGS Type: {} ===", dggs);

        println!("Global");
        let result = generator.whole_earth(dggs.to_string(), 2, false, None);
        println!(
            "{:?} \nGenerated {} cells",
            result.cells,
            result.cells.len()
        );

        println!("Global with Bbox");
        let result = generator.whole_earth(dggs.to_string(), 2, false, bbox.clone());
        println!(
            "{:?} \nGenerated {} cells",
            result.cells,
            result.cells.len()
        );

        println!("Point");
        let result = generator.from_point(dggs.to_string(), 6, pnt, false);
        println!(
            "{:?} \nGenerated {} cells",
            result.cells,
            result.cells.len()
        );

        println!("Subzones of {}", cell_id);
        let result = generator.coarse_cells(dggs.to_string(), 6, cell_id, 3, false);
        println!(
            "{:?} \nGenerated {} cells",
            result.cells,
            result.cells.len()
        );
    }
}
````

Instead of printing out the length of `grid.cells.len()` you can also print out the struct itself.
