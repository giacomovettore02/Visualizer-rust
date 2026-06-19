
use std::sync::{Arc, Mutex};

use robotics_lib::energy::Energy;
use robotics_lib::interface::{robot_map, robot_view};
use robotics_lib::runner::{backpack::BackPack, Runnable};
use robotics_lib::world::{
    coordinates::Coordinate,
    environmental_conditions::{EnvironmentalConditions, WeatherType},
    tile::{Content, Tile, TileType},
    World,
};


pub type Infos = (
    Vec<Vec<[f32; 4]>>,         // A 2D matrix of color data for tile types
    Vec<Vec<[f32; 4]>>,         // A 2D matrix of color data for tile contents
    (usize, usize),             // Current robot coordinates
    Vec<Vec<Option<Tile>>>,     // A 2D matrix of the visible/known world
    String,                     // Backpack content as text
    usize,                      // Some integer metric (e.g., energy)
    f32,                        // Score or another float metric
    WeatherType,                // Current weather condition
    String
);


/// Sentient trait extends Runnable with a custom 'act' method describing the robot's decision-making.
pub trait Sentient: Runnable {
    fn act(&mut self, world: &mut World);
}

/// Visualizable trait provides getters for Arc<Mutex<...>> fields, so external code (GUI, logging, etc.)
/// can observe the robot’s internal state in a thread-safe manner.
pub trait Visualizable {
    fn get_current_robot_map(&self) -> Arc<Mutex<Option<Vec<Vec<Option<Tile>>>>>>;
    fn get_current_robot_view(&self) -> Arc<Mutex<Vec<Vec<Option<Tile>>>>>;
    fn get_current_robot_backpack(&self) -> Arc<Mutex<String>>;
    fn get_score(&self) -> Arc<Mutex<f32>>;
    fn get_current_robot_coordinates(&self) -> Arc<Mutex<(usize, usize)>>;
    fn get_current_energy(&self) -> Arc<Mutex<usize>>;
    fn get_environmental_conditions(&self) -> Arc<Mutex<EnvironmentalConditions>>;
}

// -------------------------------------------------------------------------------------------------
// Functions for mapping tile types and tile contents to colors.
// These small helpers return RGBA or piston-friendly float arrays of RGBA values.
// -------------------------------------------------------------------------------------------------

/// Returns RGBA as a tuple (u8, u8, u8, u8) for a given TileType.
pub(crate) fn match_color_to_type(tile_type: &TileType) -> (u8, u8, u8, u8) {
    match tile_type {
        TileType::Grass         => (0, 255, 0, 255),
        TileType::Street        => (0, 0, 0, 255),
        TileType::ShallowWater  => (0, 0, 255, 255),
        TileType::DeepWater     => (0, 0, 128, 255),
        TileType::Sand          => (255, 255, 0, 255),
        TileType::Hill          => (255, 128, 0, 255),
        TileType::Mountain      => (128, 128, 128, 255),
        TileType::Wall          => (255, 128, 0, 255),
        TileType::Teleport(_)   => (255, 0, 255, 255),
        TileType::Lava          => (255, 0, 0, 255),
        TileType::Snow          => (255, 255, 255, 255),
    }
}

/// Returns RGBA as a tuple (u8, u8, u8, u8) for a given Content type.
pub(crate) fn match_color_to_content(content: &Content) -> (u8, u8, u8, u8) {
    match content {
        Content::Rock(_)       => (112, 128, 144, 255),
        Content::Tree(_)       => (0, 100, 0, 255),
        Content::Garbage(_)    => (0, 0, 0, 255),
        Content::Fire          => (255, 0, 0, 255),
        Content::Coin(_)       => (255, 215, 0, 255),
        Content::Bin(_)        => (70, 130, 180, 255),
        Content::Crate(_)      => (255, 128, 0, 255),
        Content::Bank(_)       => (128, 128, 128, 255),
        Content::Water(_)      => (173, 216, 230, 255),
        Content::Market(_)     => (255, 0, 255, 255),
        Content::Fish(_)       => (64, 224, 208, 255),
        Content::Building      => (204, 85, 0, 255),
        Content::Bush(_)       => (50, 205, 50, 255),
        Content::JollyBlock(_) => (255, 192, 203, 255),
        Content::Scarecrow     => (160, 82, 45, 255),
        Content::None          => (0, 0, 0, 0),
    }
}

/// Converts a TileType to a float-based RGBA array for piston rendering.
pub fn match_color_to_type_piston(tile_type: &TileType) -> [f32; 4] {
    let rgb = match_color_to_type(tile_type);
    [
        rgb.0 as f32 / 255.0,
        rgb.1 as f32 / 255.0,
        rgb.2 as f32 / 255.0,
        rgb.3 as f32 / 255.0,
    ]
}

/// Converts Content to a float-based RGBA array for piston rendering.
pub fn match_content_color_to_type_piston(content: &Content) -> [f32; 4] {
    let rgb = match_color_to_content(content);
    [
        rgb.0 as f32 / 255.0,
        rgb.1 as f32 / 255.0,
        rgb.2 as f32 / 255.0,
        rgb.3 as f32 / 255.0,
    ]
}

// -------------------------------------------------------------------------------------------------
// Matrix conversion helpers: build color matrices from tile data (either type or content).
// -------------------------------------------------------------------------------------------------

/// Converts an entire tile matrix into a color matrix (for tile types) and stores it in an Arc<Mutex<...>>.
pub fn convert_to_color_matrix(
    tile_matrix: &Option<Vec<Vec<Option<Tile>>>>,
    color_matrix: &Arc<Mutex<Vec<Vec<[f32; 4]>>>>,
) {
    if let Some(rows) = tile_matrix {
        let mut guard = color_matrix.lock().unwrap();
        for (i, row) in rows.iter().enumerate() {
            for (j, tile_option) in row.iter().enumerate() {
                let color = match tile_option {
                    Some(tile) => match_color_to_type_piston(&tile.tile_type),
                    None => [0.0, 0.0, 0.0, 0.0],
                };
                guard[j][i] = color;
            }
        }
    }
}

/// Converts an entire tile matrix into a color matrix (for tile contents) and stores it in an Arc<Mutex<...>>.
pub fn convert_content_to_color_matrix(
    tile_matrix: &Option<Vec<Vec<Option<Tile>>>>,
    color_matrix: &Arc<Mutex<Vec<Vec<[f32; 4]>>>>,
) {
    if let Some(rows) = tile_matrix {
        let mut guard = color_matrix.lock().unwrap();
        for (i, row) in rows.iter().enumerate() {
            for (j, tile_option) in row.iter().enumerate() {
                let color = match tile_option {
                    Some(tile) => match_content_color_to_type_piston(&tile.content),
                    None => [0.0, 0.0, 0.0, 0.0],
                };
                guard[j][i] = color;
            }
        }
    }
}

/// Builds a 3x3 color matrix from the robot's 3x3 local 'view' (tile types).
pub fn convert_robot_view_to_color_matrix(view: &Vec<Vec<Option<Tile>>>) -> Vec<Vec<[f32; 4]>> {
    let mut color_matrix = vec![vec![[0.0, 0.0, 0.0, 0.0]; 3]; 3];
    for (i, row) in view.iter().enumerate() {
        for (j, tile_option) in row.iter().enumerate() {
            let color = match tile_option {
                Some(tile) => match_color_to_type_piston(&tile.tile_type),
                None => [105.0 / 255.0, 105.0 / 255.0, 105.0 / 255.0, 1.0],
            };
            color_matrix[j][i] = color;
        }
    }
    color_matrix
}

/// Builds a 3x3 color matrix from the robot's 3x3 local 'view' (tile contents).
pub fn convert_robot_content_view_to_color_matrix(view: &Vec<Vec<Option<Tile>>>) -> Vec<Vec<[f32; 4]>> {
    let mut color_matrix = vec![vec![[0.0, 0.0, 0.0, 1.0]; 3]; 3];
    for (i, row) in view.iter().enumerate() {
        for (j, tile_option) in row.iter().enumerate() {
            let color = match tile_option {
                Some(tile) => match_content_color_to_type_piston(&tile.content),
                None => [0.0, 0.0, 0.0, 0.0],
            };
            color_matrix[j][i] = color;
        }
    }
    color_matrix
}


pub fn backpack_to_text(backpack: &BackPack) -> String {
    if backpack.get_size() > 0 && !backpack.get_contents().is_empty() {
        let mut text = format!("Backpack (Size: {}):  ", backpack.get_size());
        for (content, qty) in backpack.get_contents() {
            if *qty > 0 {
                text += &format!("{}: {}  ", content, qty);
            }
        }
        text
    } else {
        "Empty backpack".to_string()
    }
}

// -------------------------------------------------------------------------------------------------
// Generic resource update function. Locks the resource and replaces it with a new value.
// -------------------------------------------------------------------------------------------------
pub fn update_resource<T>(resource: &Mutex<T>, new_value: T) -> Result<(), String> {
    match resource.lock() {
        Ok(mut lock) => {
            *lock = new_value;
            Ok(())
        }
        Err(_) => Err("Mutex was poisoned".to_string()),
    }
}

// -------------------------------------------------------------------------------------------------
// Robot state update helpers: each uses 'update_resource' to lock and modify shared data safely.
// -------------------------------------------------------------------------------------------------

/// Updates the robot's current view field using 'robot_view()' from the robotics_lib.
pub fn update_robot_view<R>(robot: &R, world: &World) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(&robot.get_current_robot_view(), robot_view(robot, world))
}

/// Updates the robot's map field using 'robot_map()' from the robotics_lib.
pub fn update_robot_map<R>(robot: &R, world: &World) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(&robot.get_current_robot_map(), robot_map(world))
}

/// Updates the robot's coordinates (row, col).
pub fn update_robot_coord<R>(robot: &R, new_coord: &Coordinate) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(
        &robot.get_current_robot_coordinates(),
        (new_coord.get_row(), new_coord.get_col()),
    )
}

/// Updates the robot's backpack by converting its contents to a string using 'backpack_to_text()'.
pub fn update_robot_backpack<R>(robot: &R, back_pack: &BackPack) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(
        &robot.get_current_robot_backpack(),
        backpack_to_text(back_pack),
    )
}

/// Updates the robot's score with a new float value.
pub fn update_robot_score<R>(robot: &R, new_score: f32) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(&robot.get_score(), new_score)
}

/// Updates the robot's energy level by extracting it from the 'Energy' struct.
pub fn update_robot_energy<R>(robot: &R, new_energy: &Energy) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(&robot.get_current_energy(), new_energy.get_energy_level())
}

/// Updates the robot's environmental conditions (weather, temperature, etc.).
pub fn update_robot_environmental_conditions<R>(
    robot: &R,
    new_env: EnvironmentalConditions,
) -> Result<(), String>
    where
        R: Visualizable + Runnable,
{
    update_resource(&robot.get_environmental_conditions(), new_env)
}
