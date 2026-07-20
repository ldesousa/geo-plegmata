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

## Raster files

The CLI utility can convert raster files (GeoTIFF, etc.) into the internal format. The output will be a Zarr file containing the encoded raster data and metadata. You can read more about the Zarr format [here](https://zarr.readthedocs.io/en/stable/).

Each level of the pyramid is stored as a separate group in the Zarr file. On each level, the raster data is stored in chunks, each corresponding to a cell of lower resolution. The chunk size is automatically determined based on the resolution of the raster. New levels can be added to the pyramid with the `add-level` command, which will create a new group in Zarr.

### Vector files

The CLI utility can convert vector files (GeoJSON, Shapefile, etc.) into the internal format. The output will be a JSON file containing the encoded geometries and metadata.

This vector output is similar to GeoJSON, but with some additional metadata:

- `dggrs`: the DGGRS used for the conversion, such as `H3`.
- `refinement_level`: the refinement level used to encode the geometries.

By default, the maximum refinement level of the target DGGRS is used, but you can specify a different level with the `--level` option.

### Viewer

The `viewer-3d` app can load the files converted by this library directly. Read the [viewer-3d README](../viewer-3d/README.md) for more information on how to build and run the viewer app.

## Example

Here is an example of how to use the CLI utility to convert a GeoTIFF file, create a pyramid, and query some statistics about the raster:

```bash
cargo run -p gp-encoding -- convert --input input.tif --output output.zarr --dggrs H3 --report
cargo run -p gp-encoding -- add-level --store output.zarr --target-level 5
cargo run -p gp-encoding -- stats --store output.zarr
cargo run -p gp-encoding -- convert --input input.geojson --output output.json --dggrs H3 --level 7
```

