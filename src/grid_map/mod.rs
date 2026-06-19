extern crate piston_window;
use piston_window::types::Color;
use piston_window::*;
use piston_window::{rectangle, Context, G2d};

use robotics_lib::world::environmental_conditions::WeatherType;
use robotics_lib::world::tile::Tile;

/// A type alias for storing the RGBA color values in a 2D matrix structure.
pub type ColorMatrix = Vec<Vec<[f32; 4]>>;

pub const MAP_SIZE: usize = 280;
pub const GRID_SIZE: (usize, usize) = (MAP_SIZE, MAP_SIZE);
pub const RECT_SIZE: f64 = 750.0 / MAP_SIZE as f64;
pub const WINDOW_SIZE: (usize, usize) = (1000, 800);

pub const ZOOM_AMOUNT: f64 = 0.35;
pub const SCROLL_AMOUNT: f64 = 5.0;

pub const ROBOT_COLOR: [f32; 4] = [0.85, 0.55, 1.0, 1.0];

/// Draws the main grid_map in an optimized way by grouping consecutive tiles of the same color.
/// It also draws the robot's current position as a rectangle on top of the grid_map.
///
/// * `matrix` - The color matrix for the background (tile types).
/// * `context` - Piston’s drawing context for transformations.
/// * `graphics` - The 2D graphics buffer to render into.
/// * `grid_size` - The overall dimensions of the grid_map.
/// * `rect_size` - The size in pixels of each cell/rectangle (before zoom and scroll).
/// * `scroll_offset` - The current offset in X/Y for panning the view.
/// * `zoom_factor` - The current zoom level.
/// * `coord_x`/`coord_y` - Robot’s coordinates (in grid_map units).
pub fn draw_optimized_grid(
    matrix: &ColorMatrix,
    context: Context,
    graphics: &mut G2d,
    grid_size: (usize, usize),
    rect_size: f64,
    scroll_offset: [f64; 2],
    zoom_factor: f64,
    coord_x: f64,
    coord_y: f64,
) {
    // Calculate visible area considering zoom and scrolling
    let visible_start_col = ((scroll_offset[0] / zoom_factor) / rect_size).max(0.0) as usize;
    let visible_start_row = ((scroll_offset[1] / zoom_factor) / rect_size).max(0.0) as usize;
    let visible_end_col = (((scroll_offset[0] + WINDOW_SIZE.0 as f64) / zoom_factor) / rect_size)
        .min(grid_size.0 as f64) as usize;
    let visible_end_row = (((scroll_offset[1] + WINDOW_SIZE.1 as f64) / zoom_factor) / rect_size)
        .min(grid_size.1 as f64) as usize;

    // Apply transformations for panning and zooming
    let transform = context
        .transform
        .trans(-scroll_offset[0], -scroll_offset[1])
        .zoom(zoom_factor);

    // Loop through each row in the visible area
    for row in visible_start_row..visible_end_row {
        let mut col = visible_start_col;
        while col < visible_end_col {
            let current_color = matrix[col][row];

            // Advance col until the color changes
            let mut next_col = col + 1;
            while next_col < visible_end_col && matrix[next_col][row] == current_color {
                next_col += 1;
            }

            // Compute rectangle dimensions for the batched columns
            let rect_x = col as f64 * rect_size * zoom_factor - scroll_offset[0];
            let rect_y = row as f64 * rect_size * zoom_factor - scroll_offset[1];
            let rect_width = (next_col - col) as f64 * rect_size * zoom_factor;

            // Draw the large merged rectangle
            rectangle(
                current_color,
                [rect_x, rect_y, rect_width, rect_size * zoom_factor],
                transform,
                graphics,
            );

            // Draw robot position as a smaller rectangle overlay
            let robot_x = coord_x * rect_size * zoom_factor - scroll_offset[0];
            let robot_y = coord_y * rect_size * zoom_factor - scroll_offset[1];
            let robot_width = rect_size * zoom_factor;
            rectangle(
                ROBOT_COLOR,
                [robot_x, robot_y, robot_width, rect_size * zoom_factor],
                transform,
                graphics,
            );

            col = next_col;
        }
    }

    // Draw the extra white rectangles outside the last columns/rows.
    let white = [1.0, 1.0, 1.0, 1.0];

    // Right-hand side
    let right_rect_x = grid_size.0 as f64 * rect_size * zoom_factor - scroll_offset[0];
    let right_rect_y = -scroll_offset[1];
    let right_rect_height = grid_size.1 as f64 * rect_size * zoom_factor;
    rectangle(
        white,
        [right_rect_x, right_rect_y, 10.0, right_rect_height],
        transform,
        graphics,
    );

    // Bottom side
    let bottom_rect_x = -scroll_offset[0];
    let bottom_rect_y = grid_size.1 as f64 * rect_size * zoom_factor - scroll_offset[1];
    let bottom_rect_width = grid_size.0 as f64 * rect_size * zoom_factor;
    rectangle(
        white,
        [bottom_rect_x, bottom_rect_y, bottom_rect_width, 10.0],
        transform,
        graphics,
    );
}

/// Draws a small 3x3 “robot view” showing tiles (rectangles) and their contents (circles).
///
/// * `rect_matrix` - Colors for tile types in the robot's 3x3 view.
/// * `circle_matrix` - Colors for tile contents in the robot's 3x3 view.
/// * `context` - Drawing context (for transformations).
/// * `graphics` - The 2D graphics buffer.
/// * `rect_size` - The pixel size for each small tile in the robot view.
pub fn draw_robot_view(
    rect_matrix: &Vec<Vec<[f32; 4]>>,
    circle_matrix: &Vec<Vec<[f32; 4]>>,
    context: Context,
    graphics: &mut G2d,
    rect_size: f64,
) {
    // Starting coordinates on the window where we place the 3x3.
    let grid_start_x = 760.0;
    let grid_start_y = 20.0;

    // Draw each cell in the 3x3
    for (row_idx, row) in rect_matrix.iter().enumerate() {
        for (col_idx, _) in row.iter().enumerate() {
            let x = grid_start_x + (col_idx as f64 * rect_size);
            let y = grid_start_y + (row_idx as f64 * rect_size);

            // Draw the rectangle representing the tile
            if let Some(&rect_color) = rect_matrix
                .get(row_idx)
                .and_then(|some_row| some_row.get(col_idx))
            {
                rectangle(
                    rect_color,
                    [x, y, rect_size, rect_size],
                    context.transform,
                    graphics,
                );
            }

            // Draw the circle representing the tile content
            let circle_radius = rect_size / 4.0;
            let circle_x = x + rect_size / 2.0 - circle_radius;
            let circle_y = y + rect_size / 2.0 - circle_radius;

            if let Some(&circle_color) = circle_matrix
                .get(row_idx)
                .and_then(|some_row| some_row.get(col_idx))
            {
                ellipse(
                    circle_color,
                    [circle_x, circle_y, circle_radius * 2.0, circle_radius * 2.0],
                    context.transform,
                    graphics,
                );
            }
        }
    }
}

/// Draws the rectangular energy bar, color-coded by energy level.
///
/// * `energy_level` - The robot’s current energy as an integer.
/// * `context` - Drawing context.
/// * `graphics` - 2D graphics buffer.
/// * `start_x`/`start_y` - Upper-left coordinates where the bar is drawn.
pub fn draw_energy_level(
    energy_level: usize,
    context: &Context,
    graphics: &mut G2d,
    start_x: f64,
    start_y: f64,
) {
    // If energy=1000, the bar is 100 pixels wide; proportionally less otherwise.
    let bar_length = (energy_level as f64 / 1000.0) * 100.0;

    // Pick color based on energy
    let color: [f32; 4] = match energy_level {
        801..=1000 => {
            let factor = (energy_level as f64 - 800.0) / 200.0;
            [
                0.0,
                ((1.0 - factor) * 139.0 / 255.0 + factor) as f32,
                0.0,
                1.0,
            ]
        }
        601..=800 => {
            let factor = (energy_level as f64 - 600.0) / 200.0;
            [
                (factor * 255.0 / 255.0) as f32,
                ((1.0 - factor) * 255.0 / 255.0) as f32,
                0.0,
                1.0,
            ]
        }
        401..=600 => [
            255.0 / 255.0,
            ((1.0 - ((energy_level as f64 - 400.0) / 200.0)) as f32) * 165.0 / 255.0,
            0.0,
            1.0,
        ],
        201..=400 => {
            let factor = (energy_level as f64 - 200.0) / 200.0;
            [
                1.0,
                (factor * 69.0 / 255.0) as f32,
                (factor * 69.0 / 255.0) as f32,
                1.0,
            ]
        }
        _ => [1.0, 0.69, 0.69, 1.0],
    };

    // Draw the energy bar
    rectangle(
        color,
        [start_x, start_y, bar_length, 10.0],
        context.transform,
        graphics,
    );
}

/// A convenience function to draw text on the screen with a chosen color, position, and string.
pub fn draw_text(
    ctx: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
    color: Color,
    pos: [u32; 2],
    text: &str,
) {
    Text::new_color(color, 15)
        .draw(
            text,
            glyphs,
            &ctx.draw_state,
            ctx.transform.trans(pos[0] as f64, pos[1] as f64),
            graphics,
        )
        .unwrap();
}

/// Draws the label "SCORE:" and the numeric score next to it.
pub fn draw_score(
    score: f32,
    context: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
) {
    draw_text(context, graphics, glyphs, [1.0; 4], [760, 720], "SCORE:");
    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [840, 720],
        &score.floor().to_string(),
    );
}

/// Draws a colored rectangle and text describing the current weather.
pub fn draw_weather(
    weather: WeatherType,
    context: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
) {
    let rect_color = match weather {
        WeatherType::Sunny           => [1.0, 1.0, 0.0, 1.0],
        WeatherType::Rainy           => [0.0, 0.0, 1.0, 1.0],
        WeatherType::Foggy           => [0.5, 0.5, 0.5, 1.0],
        WeatherType::TropicalMonsoon => [0.0, 1.0, 0.0, 1.0],
        WeatherType::TrentinoSnow    => [1.0, 1.0, 1.0, 1.0],
    };

    // The rectangle on the right-side column
    let rect_x = 760.0;
    let rect_y = 270.0;
    let rect_width = 80.0;
    let rect_height = 40.0;
    rectangle(
        rect_color,
        [rect_x, rect_y, rect_width, rect_height],
        context.transform,
        graphics,
    );

    let weather_text = match weather {
        WeatherType::Sunny           => "SUNNY and bright!",
        WeatherType::Rainy           => "RAINY, keep your umbrella!",
        WeatherType::Foggy           => "FOGGY, hard to see!",
        WeatherType::TropicalMonsoon => "TROPICAL MONSOON, very hot!",
        WeatherType::TrentinoSnow    => "TRENTINO SNOW, freezing cold!",
    };

    draw_text(context, graphics, glyphs, [1.0; 4], [760, 250], "WEATHER:");
    draw_text(context, graphics, glyphs, [1.0; 4], [840, 250], weather_text);
}

/// Draws the text “ENERGY:” plus the energy value, and calls `draw_energy_level` for the bar.
pub fn draw_energy(
    energy: usize,
    context: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
) {
    draw_text(context, graphics, glyphs, [1.0; 4], [760, 650], "ENERGY:");
    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [845, 650],
        &energy.to_string(),
    );
    draw_energy_level(energy, context, graphics, 760.0, 660.0);
}

/// Draws three lines of text describing each row in the robot’s 3x3 view.
///
/// * `vec1`, `vec2`, `vec3` - Optional references to a vector of tiles for each row.
/// * `context`, `graphics`, `glyphs` - Standard Piston drawing objects.
/// * `starting_text_y` - The vertical offset where the text is printed.
pub fn draw_texts(
    vec1: &Option<&Vec<Option<Tile>>>,
    vec2: &Option<&Vec<Option<Tile>>>,
    vec3: &Option<&Vec<Option<Tile>>>,
    context: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
    starting_text_y: u32,
) {
    let start_x: u32 = 50;
    let start_y: u32 = starting_text_y + 35;
    let offset: u32 = 25;

    if let Some(row) = vec1 {
        draw_text(
            context,
            graphics,
            glyphs,
            [1.0; 4],
            [start_x, start_y + offset],
            &create_text_view(row),
        );
    }

    if let Some(row) = vec2 {
        draw_text(
            context,
            graphics,
            glyphs,
            [1.0; 4],
            [start_x, start_y + offset * 2],
            &create_text_view(row),
        );
    }

    if let Some(row) = vec3 {
        draw_text(
            context,
            graphics,
            glyphs,
            [1.0; 4],
            [start_x, start_y + offset * 3],
            &create_text_view(row),
        );
    }
}

/// Creates a single string by joining each tile’s content in a vector.
///
/// * `vec` - A vector of optional Tiles.
pub fn create_text_view(vec: &Vec<Option<Tile>>) -> String {
    let mut result = String::new();
    for maybe_tile in vec {
        let content_str = match maybe_tile {
            Some(tile) => format!("{:?} ", tile.content),
            None => "[x]".to_string(),
        };
        // Give each token some spacing so columns align more neatly
        result += &format!("{:<10} ", content_str);
    }
    result
}
