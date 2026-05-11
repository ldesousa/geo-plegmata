
## For contributors
For people who want to contribute but lack the knowledge of working with a rust project. If you haven't used rust in the past, please follow these instructions.

### Install Rust and Cargo

Cargo comes bundled with Rust. The easiest way to install both is with rustup
```bash
curl https://sh.rustup.rs -sSf | sh
```

Then reload your shell and confirm installation:
```bash
rustc --version
cargo --version
```

### Quick setup:
```bash
cd gp-proj
cargo build
cargo run --example basic
```

If you want more detailed instructions, read below.

### Build/Run Dependencies

Make sure [DGGRID](https://github.com/sahrk/DGGRID) is compiled and available on your system. Remember the path where the `dggrid` executable is, or add `dggrid` to your `$PATH`.

### Build the project
After you clone the repo, you can either build from the root of the repo or accessing one of the subcrates you want to work with and build from there. 
As an example:
```bash
cd gp-proj
cargo build
```

To build in release mode (optimized):
```bash
cargo build --release
```

To build a specific crate without going to a specific directory:
```bash
cargo build -p gp-proj
```

You can list all workspace members with:
```bash
cargo metadata --no-deps --format-version=1 | jq '.packages[].name'
```

Note: If you want a quicker way to check for errors when you are building, run `cargo check` frequently — it’s faster than a full build and helps catch errors early. Or you can use [https://dystroy.org/bacon/](bacon) instead of `cargo build`.

To clean all built artifacts and dependencies, run:
```bash
cargo clean
```

To update dependencies, run:
```bash
cargo update
```

### Adding a dependency from crates.io
If your Cargo.toml doesn't already have a [dependencies] section, add it. Then list the crate name and version that you would like to use. This example adds a dependency of the time crate:
```rust
[dependencies]
time = "0.1.12"
```
Re-run `cargo build`, and Cargo will fetch the new dependencies and all of their dependencies, compile them all, and update the Cargo.lock. Our Cargo.lock contains the exact information about which revision of all of these dependencies we used.

### Testing
To run all tests for all crates:
```bash
cargo test
```

To test only one crate:
```bash
cargo test -p utils
```

Cargo can run your tests with the cargo test command. Cargo looks for tests to run in two places: in each of your src files and any tests in tests/. Tests in your src files should be unit tests, and tests in tests/ should be integration-style tests. As such, you’ll need to import your crates into the files in tests.
You can also run a specific test by passing a filter:
```bash
cargo test foo
```
