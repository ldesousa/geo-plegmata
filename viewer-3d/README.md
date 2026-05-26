# Visualization App

This is a simple visualization app for the GP encoding project. It allows to visualize the results of the GP encoding library directly in a desktop application. The visualization app is built using Tauri, and it uses deck.gl for rendering the grids.


## Building and Running

To build and run the visualization app, follow these steps:

1. Make sure to install all the Tauri dependencies on your system. You can find the prerequisites [here](https://tauri.app/start/prerequisites/).

2. Install the node dependencies by running the following command in the terminal:

```bash
pnpm install
```

3. Run the development app with the following command:

```bash
pnpm tauri dev
```

If you want to build the app for production, use the following command:

```bash
pnpm tauri build
```

The built app will be located in the `src-tauri/target/release` directory. You can run the executable file to launch the visualization app.