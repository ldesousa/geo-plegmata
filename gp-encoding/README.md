# gp-encoding

This crate contains the tools used by GeoPlegma for encoding geospatial data into its internal format based on Zarr. It includes both the library code and a CLI utility.

The `viewer-3d` crate is able to read the encoded data from this crate and provides a way to visualize it.

## Building

Build the crate directly with `cargo`:

```bash
cargo build -p gp-encoding
```

## Usage

To run the CLI utility, use:

```bash
cargo run -p gp-encoding -- <args>
```

To get the list of available commands and options, use:

```bash
cargo run -p gp-encoding -- --help
```

## Example

Here is an example of how to use the CLI utility to convert a GeoTIFF file, create a pyramid, and query some statistics about the raster:

```bash
cargo run -p gp-encoding -- convert-geotiff --input input.tif --output output.zarr --dggrs H3 --report
cargo run -p gp-encoding -- add-level --store output.zarr --target-level 5
cargo run -p gp-encoding -- stats --store output.zarr
```

## Benchmarking

The input files for the benchmarks should be put in `gp-encoding/benches/files`. Every file in that directory will be converted using the library. The tests will measure time, accuracy and disk usage, with reports being generated in `gp-encoding/benches/bench_out`.

To run the benchmarks, use:

```bash
cargo bench --bench convert
```

