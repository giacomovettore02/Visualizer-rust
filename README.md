# Visualizer-rust

`Visualizer-rust` is a Rust-based visualization project built to display the state of an autonomous robot exploring a procedurally generated world. The code focuses on turning simulation data into an interactive desktop view: map tiles, visible robot area, backpack contents, weather, time, and energy are all rendered live in a custom UI built with `piston_window`.

This repository is presented as a portfolio project. The goal of this cleanup is to show the actual code and project structure clearly, without shipping generated build artifacts, editor metadata, or private machine-specific credentials.

## What This Project Shows

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

## Important Build Notes

This project is not a fully standalone public crate at the moment, and that is intentional to state clearly:

- It depends on crates fetched from a private `kellnr` registry
- It also references a local path dependency: `andrea_ai = { path = "../../robot_ai" }`
- Because of those private/local dependencies, a fresh public clone may not compile without access to the original environment

For portfolio purposes, the repository is still valuable because it shows:

- code organization
- rendering logic
- synchronization patterns
- integration style
- project scale and complexity

## Why The Repo Was Cleaned Up

The original repository contained generated build output under `target/`, editor-specific project files, and machine-specific Cargo authentication details. Those files do not help a recruiter evaluate the code and make the repository unnecessarily large and noisy.

This version keeps the focus on the source code and project intent.

## Tech Stack

- Rust 2021
- `piston_window`
- `robotics_lib`
- `noise`
- `image`
- `rayon`
- `crossbeam-channel`
- `rodio`
- `clap`

## Honest Limitations

- The project is tightly coupled to its original robotics ecosystem
- Some dependencies are private or local-only
- No extra refactor was done here to make the code more public-package-friendly, because the goal was to preserve the original implementation and only improve repository quality

## Recruiter-Facing Summary

If you are reviewing this repository as part of a portfolio, the strongest signals are:

- the project is larger than a tutorial-sized app
- the author worked with rendering, concurrency, and shared state
- the code integrates simulation data into a usable visualization tool
- the repository cleanup keeps attention on the real engineering work instead of generated output
