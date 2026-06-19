use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

// Piston imports for visualization
use piston_window::{
    clear, Button, G2d, Glyphs, Key, OpenGL, PistonWindow, PressEvent, ReleaseEvent, Size,
    UpdateEvent, WindowSettings,
};

// External generator for the world
use rustafariani_world_gen::worldgen::generator::WorldGenerator;

// Robotics and visualization crates
use robotics_lib::runner::{Robot, Runner};
use robotics_lib::world::environmental_conditions::{EnvironmentalConditions, WeatherType};
use robotics_lib::world::environmental_conditions::WeatherType::*;
use robotics_lib::world::tile::Tile;
use Visualizer::common::{
    Infos, Visualizable, convert_content_to_color_matrix, convert_robot_content_view_to_color_matrix,
    convert_robot_view_to_color_matrix, convert_to_color_matrix,
};
use Visualizer::grid_map::{
    create_text_view, draw_energy, draw_optimized_grid, draw_robot_view,
    draw_text, draw_texts, draw_weather, MAP_SIZE, RECT_SIZE, SCROLL_AMOUNT, WINDOW_SIZE, ZOOM_AMOUNT,
};

/// The default path for the font
const DEFAULT_FONT_PATH: &str = "../../font/font.otf";

/// Synchronized dimension with the grid_map's MAP_SIZE
pub const MAP_DIM: usize = MAP_SIZE;

fn main() {
    // -------------------------------------------------------------------------
    // 1) Construct the AI Robot and parse any CLI arguments
    // -------------------------------------------------------------------------
    use andrea_ai::andrea_robot::AndreaRobot;
    use clap::Parser;

    let app_args = andrea_ai::andrea_robot::Args::parse();
    let andrea_bot = AndreaRobot::new(Robot::new(), Arc::new(Mutex::new(0)), app_args);

    // -------------------------------------------------------------------------
    // 2) Create channels to send map/view/score data to the visualizer
    // -------------------------------------------------------------------------
    let (tx_matrix, rx_matrix) = mpsc::channel();

    // Cloned references to the robot’s internal shared state
    let current_robot_map = andrea_bot.get_current_robot_map().clone();
    let current_robot_view = andrea_bot.get_current_robot_view().clone();
    let current_robot_backpack = andrea_bot.get_current_robot_backpack().clone();
    let score_arc = andrea_bot.get_score().clone(); // Score is still tracked internally
    let conditions_arc = andrea_bot.get_environmental_conditions().clone();
    let current_robot_coordinates = andrea_bot.get_current_robot_coordinates().clone();
    let current_robot_energy = andrea_bot.get_current_energy().clone();

    // -------------------------------------------------------------------------
    // 3) Spawn a background thread to run the simulation logic (Runner)
    // -------------------------------------------------------------------------
    thread::spawn(move || {
        // Prepare a generator with custom environment conditions
        let mut generator = WorldGenerator::new()
            .set_size(MAP_SIZE)
            .set_weather_conditions(
                EnvironmentalConditions::new(
                    &[Sunny, Rainy, Foggy, TropicalMonsoon, TrentinoSnow],
                    15,
                    12,
                )
                    .unwrap(),
            );

        let iteration_counter = andrea_bot.iterations.clone();
        let mut runner_session = Runner::new(Box::new(andrea_bot), &mut generator);

        loop {
            match runner_session {
                Ok(ref mut runner) => {
                    let _ = runner.game_tick();
                    // Stop after enough iterations
                    if *iteration_counter.lock().unwrap() > 2000 {
                        break;
                    }
                }
                Err(e) => {
                    println!("Runner encountered an error: {:?}", e);
                    break;
                }
            }
        }
    });

    // -------------------------------------------------------------------------
    // 4) Create the Piston window for visualization
    // -------------------------------------------------------------------------
    let window_size = Size::from((WINDOW_SIZE.0 as u32, WINDOW_SIZE.1 as u32));
    println!("Initializing the visualization window...");

    let mut app_window: PistonWindow = WindowSettings::new("Visualizer 2.0", window_size)
        .exit_on_esc(true)
        .resizable(false)
        .graphics_api(OpenGL::V3_2)
        .build()
        .unwrap();

    let mut glyphs = match app_window.load_font(DEFAULT_FONT_PATH) {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("Could not load the font: {}", e);
            None
        }
    };

    // Prepare color matrices that will be updated in the sender thread
    let tile_color_matrix = Arc::new(Mutex::new(vec![
        vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM];
        MAP_DIM
    ]));

    let content_color_matrix = Arc::new(Mutex::new(vec![
        vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM];
        MAP_DIM
    ]));

    // -------------------------------------------------------------------------
    // 5) Spawn a thread to produce updated info and send it to the visualization
    // -------------------------------------------------------------------------
    let tx_clone = tx_matrix.clone();
    thread::spawn(move || {
        loop {
            // Lock each resource carefully and clone/deref the data we need:

            // 1) Robot energy (usize)
            let updated_energy = match current_robot_energy.lock() {
                Ok(guard) => *guard,
                Err(_) => 0, // fallback
            };

            // 2) Robot score (f32)
            let updated_score = match score_arc.lock() {
                Ok(guard) => *guard,
                Err(_) => 0.0,
            };

            // 3) Full map (Option<Vec<Vec<Option<Tile>>>>)
            let updated_map = match current_robot_map.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => None,
            };

            // 4) Robot coordinates ((usize, usize))
            let updated_coord = match current_robot_coordinates.lock() {
                Ok(guard) => *guard,
                Err(_) => (0, 0),
            };

            // 5) Robot view (Vec<Vec<Option<Tile>>>)
            let updated_view = match current_robot_view.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => vec![vec![None; 3]; 3],
            };

            // 6) Backpack content (String)
            let updated_backpack = match current_robot_backpack.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => String::new(),
            };

            // 7) Environment (weather/time)
            let (updated_weather, updated_time) = match conditions_arc.lock() {
                Ok(guard) => (guard.get_weather_condition(), guard.get_time_of_day_string()),
                Err(_) => (WeatherType::Sunny, "--:--".to_string()),
            };

            // Update the color matrices for the map based on tile type/content
            convert_content_to_color_matrix(&updated_map, &content_color_matrix);
            convert_to_color_matrix(&updated_map, &tile_color_matrix);

            // 8) Copy the color matrices out (Vec<Vec<[f32; 4]>>)
            let tile_colors = match tile_color_matrix.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],
            };

            let content_colors = match content_color_matrix.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],
            };

            // Send the fully-cloned data to the visualization
            let info_tuple = (
                tile_colors,      // 0) tile color matrix
                content_colors,   // 1) content color matrix
                updated_coord,    // 2) (row, col)
                updated_view,     // 3) 3x3 local robot view
                updated_backpack, // 4) backpack string
                updated_energy,   // 5) energy
                updated_score,    // 6) score
                updated_weather,  // 7) WeatherType
                updated_time,     // 8) time string
            );

            if tx_clone.send(info_tuple).is_err() {
                eprintln!("Failed to send data through the channel.");
            }

            thread::sleep(Duration::from_secs_f64(0.2));
        }
    });

    // -------------------------------------------------------------------------
    // 6) Prepare variables for rendering
    // -------------------------------------------------------------------------
    let mut current_info: Infos = (
        vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],  // tile colors
        vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],  // content colors
        (0, 0),                                             // robot coords
        vec![vec![None; 3]; 3],                             // 3x3 view
        String::new(),                                      // backpack
        0,                                                  // energy
        0.0,                                                // score (tracked, but not displayed)
        WeatherType::Sunny,                                 // weather
        String::new()                                       // time
    );

    let mut scroll_offset = [0.0, 0.0];
    let mut zoom_factor = 1.0;

    // For toggling displays
    let mut show_robot_view = true;
    let mut show_info_text = true;

    // For continuous movement via pressed keys
    let mut pressing_left = false;
    let mut pressing_right = false;
    let mut pressing_up = false;
    let mut pressing_down = false;

    // -------------------------------------------------------------------------
    // 7) Main event loop for drawing
    // -------------------------------------------------------------------------
    while let Some(event) = app_window.next() {
        // Non-blocking attempt to receive updated data from the channel
        if let Ok(latest_data) = rx_matrix.try_recv() {
            current_info = latest_data;
        }

        // Build a text to show robot coordinates in the UI
        let coord_text = format!(
            "Robot coords: ({}, {})",
            current_info.2 .1, current_info.2 .0
        );
        let coord_as_f64 = (current_info.2 .1 as f64, current_info.2 .0 as f64);

        // ---------------------------------------------------------------------
        // Event: Key press
        // ---------------------------------------------------------------------
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::Up => {
                    scroll_offset[1] -= SCROLL_AMOUNT;
                    pressing_up = true;
                }
                Key::Down => {
                    scroll_offset[1] += SCROLL_AMOUNT;
                    pressing_down = true;
                }
                Key::Left => {
                    scroll_offset[0] -= SCROLL_AMOUNT;
                    pressing_left = true;
                }
                Key::Right => {
                    scroll_offset[0] += SCROLL_AMOUNT;
                    pressing_right = true;
                }
                Key::V => {
                    show_robot_view = !show_robot_view;
                }
                Key::T => {
                    show_info_text = !show_info_text;
                }
                Key::Minus => {
                    zoom_factor -= ZOOM_AMOUNT;
                    if zoom_factor < 0.1 {
                        zoom_factor = 0.1;
                    }
                }
                Key::Plus | Key::Equals => {
                    zoom_factor += ZOOM_AMOUNT;
                }
                _ => {}
            }
        }

        // ---------------------------------------------------------------------
        // Event: Update (continuous hold)
        // ---------------------------------------------------------------------
        event.update(|_| {
            if pressing_left {
                scroll_offset[0] -= SCROLL_AMOUNT;
            }
            if pressing_right {
                scroll_offset[0] += SCROLL_AMOUNT;
            }
            if pressing_down {
                scroll_offset[1] += SCROLL_AMOUNT;
            }
            if pressing_up {
                scroll_offset[1] -= SCROLL_AMOUNT;
            }
        });

        // ---------------------------------------------------------------------
        // Event: Key release (stop continuous movement)
        // ---------------------------------------------------------------------
        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::Up => pressing_up = false,
                Key::Down => pressing_down = false,
                Key::Left => pressing_left = false,
                Key::Right => pressing_right = false,
                _ => {}
            }
        }

        // ---------------------------------------------------------------------
        // Rendering
        // ---------------------------------------------------------------------
        app_window.draw_2d(&event, |context, graphics, device| {
            // Clear background
            clear([0.0, 0.0, 0.0, 1.0], graphics);

            // Optionally draw the 3x3 robot view
            if show_robot_view {
                let tile_matrix_3x3 = convert_robot_view_to_color_matrix(&current_info.3);
                let content_matrix_3x3 = convert_robot_content_view_to_color_matrix(&current_info.3);
                draw_robot_view(&tile_matrix_3x3, &content_matrix_3x3, context, graphics, 50.0);
            }

            // Draw the main grid
            draw_optimized_grid(
                &current_info.0,  // tile colors
                context,
                graphics,
                (MAP_DIM, MAP_DIM),
                RECT_SIZE,
                scroll_offset,
                zoom_factor,
                coord_as_f64.0,
                coord_as_f64.1,
            );

            // Optionally draw HUD text
            if show_info_text {
                if let Some(ref mut glyphs) = glyphs {
                    // Coordinates
                    draw_text(&context, graphics, glyphs, [1.0; 4], [50, 785], &coord_text);

                    // Robot view textual breakdown
                    draw_texts(
                        &current_info.3.get(0),
                        &current_info.3.get(1),
                        &current_info.3.get(2),
                        &context,
                        graphics,
                        glyphs,
                        785,
                    );

                    // Backpack
                    draw_text(
                        &context,
                        graphics,
                        glyphs,
                        [1.0; 4],
                        [50, 30 + 785 + 25 * 5],
                        current_info.4.as_str(),
                    );

                    // Energy
                    draw_energy(current_info.5, &context, graphics, glyphs);

                    // Time
                    draw_text(&context, graphics, glyphs, [1.0; 4], [760, 400], "TIME:");
                    draw_text(
                        &context,
                        graphics,
                        glyphs,
                        [1.0; 4],
                        [840, 400],
                        current_info.8.as_str(),
                    );

                    // --- Score Display Removed Here ---

                    // "Robot View" label
                    draw_text(&context, graphics, glyphs, [1.0; 4], [760, 20], "Robot View");

                    // Weather
                    draw_weather(current_info.7, &context, graphics, glyphs);

                    // Flush text
                    glyphs.factory.encoder.flush(device);
                }
            }
        });
    }
}
