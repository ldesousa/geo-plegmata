# GeoPlegma

[![Stable](https://img.shields.io/badge/docs-stable-blue.svg)](https://geoplegma.github.io/GeoPlegma/stable/)
[![Dev](https://img.shields.io/badge/docs-dev-blue.svg)](https://geoplegma.github.io/GeoPlegma/dev/)
[![Build Status](https://github.com/GeoPlegma/GeoPlegma/actions/workflows/CI.yml/badge.svg?branch=master)](https://github.com/GeoPlegma/GeoPlegma/actions/workflows/CI.yml?query=branch%3Amaster)
=======
GeoPlegma
===========

## Copyright

Copyright (c) 2025 contributors to the GeoPlegma project. All rights reserved. Any use of this software constitutes full acceptance of all terms of its licence.

## Overview

This project aims to be an harbour for individuals, companies and other projects developing software implementing or using Discrete Global Grid Systems (DGGS). It is meant as an accretion point, around which collaboration synergies can accelerate the development of modern Geographic Information Systems (GIS), multiplying benefits to those contributing.

Ambitious? Certainly. Necessary? Absolutely. DGGS software has existed for more than twenty years, but is yet to have the impact on GIS (and GeoSciences at large) it should have. Dispersion of effort and objectives has been a major element in this delay.

The abstractions proposed by GeoPlegma are meant to not only accelerate development, but also facilitate the development of tools and interfaces to end users. Whereas so far the multitude of different DGGS and accompanying software have been a hurdle to anyone wishing to adopt them, GeoPlegma intendeds to create a common source code lexicon. Independently of the kind of DGGS the user wishes to use, or for what purpose, tools such as a Python API or a web-based display should present the same ease of access and use.

### The name


The term *geo* is Greek for Earth. Since DGGSs are primarily conceived as grids on the Earth's surface, it seems appropriate to use the Greek term for grid, *plegma*, to compose the name. In the beginning of the project the term *plegmata*, plural for *plegma*, was used but dropped for simplicity. 

## Licence

This project is Licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) LICENSE-APACHE or the [MIT license](http://opensource.org/licenses/MIT), at your discretion. All contents herewith may not be copied, modified, or distributed except according to the terms defined in the licence chosen. Refer to the files [LICENCE-APACHE.txt](LICENCE-APACHE.txt) and [LICENCE-MIT.txt](LICENCE-MIT.txt) for details.

# The Pillars

GeoPlegma starts by offering a collection of interfaces specifying the behaviour of programmes and code libraries wishing to "join in" the effort. They can be seen as an abstract layer setting a **contract** between different programmes. These interfaces are largely conformant to the [OGC API for DGGS](https://ogcapi.ogc.org/dggs/).

At present three pillars are part of the core design: (i) Discrete Global Grid Reference Systems (DGGRS), (ii) Data Structures and (iii) Encoding/Abstraction. The aim is for any programme implementing one of these pillars to benefit seamlessly from any functionality offered by any programme implementing one of the others.

## DGGRS

A library or programme implementing a Discrete Global Grid Reference System offers the basic functionalities to locate any position on the Earth's surface with a global grid. It translates latitude and longitude coordinates into grid cell identifiers and grid cell topologies.

Kevin Sahr first observed the capacity of a DGGS as a geo-spatial reference system in the work titled "[Location coding on icosahedral aperture 3 hexagon discrete global grids](https://doi.org/10.1016/j.compenvurbsys.2007.11.005)". By complementing the topology of a DGGS with a function mapping zones to unique identifiers, a DGGS is able to locate any location on the Earth's surface, at any desired spatial resolution.

The DGGRS interface is meant as the connection point to existing libraries, in particular [DGGRID](https://github.com/sahrk/DGGRID), but also H3, S2 and more.

## Data Strucutres

Data structures based on a DGGS bring GeoPlegma closer to what users may expect from a GIS.

### Coverage

In simple terms, a coverage is a function mapping geo-spatial coordinates into values, usually representing some phenomenon continuous in space. With DGGS a coverage becomes a function mapping grid cell identifiers into values.

A Coverage may encompass the complete surface of the Earth, or just a segment thereof. The spatial extent of the coverage is one of the elements of its meta-data, along with the identification of the underlying DGGRS.

Coverages are often organised into blocks, sub-segments of its extent that facilitate their management in memory and encoding. The OGC DGGS API defines the concept of _Zone_, that largely overlaps with that of block.

In its meta-data the Coverage must identify the DGGRS resolution of its zones, as well as the resolution of is blocks (or zones).

### Vector

The Vector concept with DGGS is also similar to that in traditional GIS, a collection of geometries to which a set of key-value pairs is associated. The only difference being with the nodes of the geometries, determined by DGGRS cell identifiers.

As with coverages, a Vector must identify in its meta-data the DGGRS and resolution determining its cell identifiers.


Encoding/Abstraction
--------------------

These assets define behaviour allowing data structures to persist. The method signatures include the encoding and abstraction of meta-data and data segments. The meta-data must clearly identify the underlying DGGRS and if necessary data types. Data access is conceived in segmented form, considering the likely case of large datasets, either spanning large areas of the globe or expressed at a high spatial resolution.  

How to use it
-------------

This library provides an interface to [DGGRID](https://github.com/sahrk/DGGRID) (and potentially other tools) to generate zones. The output of the three public functions is a `Zones` struct with the cell ID and an vector of coordinates that describes the cell polygon using the [geo](https://github.com/georust/geo) primitive [Polygon](https://docs.rs/geo/latest/geo/geometry/struct.Polygon.html).

## Requirements

Make sure [DGGRID](https://github.com/sahrk/DGGRID) is compiled and available on your system. Remember the path where the `dggrid` executable is, or add `dggrid` to your `$PATH`.

## Building and testing
> **_NOTE:_**  If you want to build the project and start developing and contributing, go [here](DEVELOPMENT.md).

## Usage Example

Check out `geoplegma/example/basic.rs` for a simple example of how to use the `geoplegma` crate of GeoPlegma.
```bash
cd geoplegma
cargo run --example basic
```

Instead of printing out the length of `result.zones.len()` you can also print out the struct itself.

To run on watch mode and see living changes:

Install cargo-watch:
```bash
cargo install cargo-watch
```
Then run (this also clears the screen):
```bash
cargo watch -c -x 'run --example basic'
```

Dependencies
------------
This crate depends on `dggal-rust` from https://github.com/GeoPlegma/dggal-rust.

# Release Workflow for bindings (Step by Step)

In GeoPlegma (Rust + bindings repo):
- Update bindings if needed.
- Tag a version (e.g. v1.2.3):
```bash
git tag v1.2.3
git push origin v1.2.3
```
- GitHub Actions builds the .node files and attaches them to the release.

In GeoPlegma-js (npm wrapper repo):
- Update package.json version to 1.2.3.
- Commit and push.
- Tag the same version:
```bash
git tag v1.2.3
git push origin v1.2.3
```
- GitHub Actions fetches the .node files from the GeoPlegma release and publishes your npm package.
