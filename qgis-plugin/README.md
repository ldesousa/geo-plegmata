# GeoPlegma QGIS Plugin

This plugin allows you to natively open and visualize GeoPlegma (`gp-encoding`) Zarr stores directly in QGIS. It relies on a native Rust extension (`geoplegma_py`) to rapidly parse the DGGS grid and load it into QGIS as a Vector Memory Layer.

## Prerequisites

To build and install the plugin from source, you will need:
- **Rust** and **Cargo**
- **Python 3**
- **Maturin** (for building the Python bindings)

You can install Maturin via pip:
```bash
pip install maturin
```

## Build the Python Bindings

Before QGIS can load the plugin, you must compile the `geoplegma_py` Rust extension for your system.

1. Open your terminal and navigate to the python bindings directory:
   ```bash
   cd gp-bindings/python
   ```
2. Build the extension using `maturin`:
   ```bash
   maturin develop
   ```
3. Copy the generated `geoplegma_py` shared library from the `target/maturin` directory into the `qgis-plugin` directory.

## Install the Plugin in QGIS

Copy the entire `qgis-plugin` folder into your QGIS plugins directory, and rename it to `geoplegma`.
   
Depending on your OS and QGIS version, the plugins directory is located at:
- **Linux**: `~/.local/share/QGIS/QGIS4/profiles/default/python/plugins/geoplegma`
- **Windows**: `%APPDATA%\QGIS\QGIS4\profiles\default\python\plugins\geoplegma`
- **macOS**: `~/Library/Application Support/QGIS/QGIS4/profiles/default/python/plugins/geoplegma`

## Enable the Plugin

1. Launch QGIS (if QGIS is already open, you must fully close and restart it for the plugin to be recognized).
2. Go to **Plugins > Manage and Install Plugins...**
3. Look for **GeoPlegma** under the *Installed* tab and check the box to enable it.
4. You will now see an "Open GeoPlegma Store" button in your toolbar
