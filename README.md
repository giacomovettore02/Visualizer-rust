# Visualizer-rust

`Visualizer-rust` is a Rust-based visualization project built to display the state of an autonomous robot exploring a procedurally generated world. The code focuses on turning simulation data into an interactive desktop view: map tiles, visible robot area, backpack contents, weather, time, and energy are all rendered live in a custom UI built with `piston_window`.

This repository is presented as a portfolio project. The goal of this cleanup is to show the actual code and project structure clearly, without shipping generated build artifacts, editor metadata, or private machine-specific credentials.

## Project contents

- Real Rust application structure split between a reusable library crate and an executable crate
- Concurrency with threads, channels, `Arc`, and `Mutex`
- Rendering and UI logic with `piston_window`
- Data transformation from simulation state into visual color matrices
- Integration with external crates and a private robotics ecosystem
- A practical, non-trivial desktop visualization workflow rather than a toy example

## Repository Structure

```text
.
|-- .cargo/
|   `-- config.toml          # Registry configuration (sanitized)
|-- bin/
|   |-- Cargo.toml           # Executable crate
|   `-- src/main.rs          # App entry point and event loop
|-- font/
|   `-- font.otf             # Font asset used by the UI
|-- src/
|   |-- common/mod.rs        # Shared traits, color conversion, robot state helpers
|   |-- grid_map/mod.rs      # Rendering helpers and UI drawing logic
|   `-- lib.rs               # Library module exports
|-- Cargo.toml               # Workspace + library crate manifest
|-- Cargo.lock
`-- README.md
```

## Architecture Overview

The codebase is organized as a small Cargo workspace:

- The root crate exposes shared visualization logic from `src/`
- The `bin/` crate contains the executable application and simulation wiring
- The executable spawns background work to advance the robot simulation while the UI thread renders the current state
- Shared robot/world data is synchronized through `Arc<Mutex<...>>`
- Render-friendly color matrices are derived from world tiles and robot view data before being drawn to the screen

## Main Features

- Large map rendering with scroll and zoom support
- Live robot position overlay
- Separate rendering for tile type and tile content
- Dedicated 3x3 robot local view
- HUD information for:
  - coordinates
  - backpack contents
  - energy
  - weather
  - time
- Keyboard toggles for changing the visualization view

## Controls

From the current implementation in [`bin/src/main.rs`](/C:/Users/giaco/Desktop/Visualizer-rust/bin/src/main.rs):

- Arrow keys: move around the rendered map
- `+` / `=`: zoom in
- `-`: zoom out
- `V`: toggle the robot local view
- `T`: toggle the text/HUD information
- `Esc`: close the window
