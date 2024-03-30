use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;
use image::{ImageBuffer, Rgb};
use imageproc::drawing::{draw_filled_circle, draw_filled_rect, draw_hollow_rect, draw_antialiased_line_segment, draw_polygon_mut, draw_filled_circle_mut, draw_cross};
use imageproc::pixelops::interpolate;
use rayon::prelude::*;
use lazy_static::lazy_static;
use std::path::Path;
use std::sync::{Arc, Mutex};
use imageproc::point::Point;
use imageproc::rect::Rect;
use noise::{NoiseFn, Perlin, Seedable};


// ===========================================================================
// Direction concepts

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

// direction offsets in terms of x,y
impl Direction {
    pub fn get_offset(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        }
    }

    // Human readable direction names
    pub fn name(&self) -> &str {
        match self {
            Direction::North => "North",
            Direction::South => "South",
            Direction::East => "East",
            Direction::West => "West",
            Direction::NorthEast => "NorthEast",
            Direction::NorthWest => "NorthWest",
            Direction::SouthEast => "SouthEast",
            Direction::SouthWest => "SouthWest",
        }
    }

    // Get the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::NorthEast => Direction::SouthWest,
            Direction::NorthWest => Direction::SouthEast,
            Direction::SouthEast => Direction::NorthWest,
            Direction::SouthWest => Direction::NorthEast,
        }
    }

    // turn right
    pub fn right(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::East => Direction::South,
            Direction::West => Direction::North,
            Direction::NorthEast => Direction::SouthEast,
            Direction::NorthWest => Direction::NorthEast,
            Direction::SouthEast => Direction::SouthWest,
            Direction::SouthWest => Direction::NorthWest,
        }
    }

    // turn left
    pub fn left(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            Direction::West => Direction::South,
            Direction::NorthEast => Direction::NorthWest,
            Direction::NorthWest => Direction::SouthWest,
            Direction::SouthEast => Direction::NorthEast,
            Direction::SouthWest => Direction::SouthEast,
        }
    }

    // get the direction from a string
    pub fn from_str(s: &str) -> Option<Direction> {
        match s {
            "North" => Some(Direction::North),
            "South" => Some(Direction::South),
            "East" => Some(Direction::East),
            "West" => Some(Direction::West),
            "NorthEast" => Some(Direction::NorthEast),
            "NorthWest" => Some(Direction::NorthWest),
            "SouthEast" => Some(Direction::SouthEast),
            "SouthWest" => Some(Direction::SouthWest),
            _ => None,
        }
    }
}

// direction tests
#[cfg(test)]
mod direction_tests {
    use super::*;

    // test direction get_offset
    #[test]
    fn test_direction_offset() {
        assert_eq!(Direction::North.get_offset(), (0, -1));
        assert_eq!(Direction::South.get_offset(), (0, 1));
        assert_eq!(Direction::East.get_offset(), (1, 0));
        assert_eq!(Direction::West.get_offset(), (-1, 0));
        assert_eq!(Direction::NorthEast.get_offset(), (1, -1));
        assert_eq!(Direction::NorthWest.get_offset(), (-1, -1));
        assert_eq!(Direction::SouthEast.get_offset(), (1, 1));
        assert_eq!(Direction::SouthWest.get_offset(), (-1, 1));
    }


    #[test]
    fn test_direction_name() {
        assert_eq!(Direction::North.name(), "North");
        assert_eq!(Direction::South.name(), "South");
        assert_eq!(Direction::East.name(), "East");
        assert_eq!(Direction::West.name(), "West");
        assert_eq!(Direction::NorthEast.name(), "NorthEast");
        assert_eq!(Direction::NorthWest.name(), "NorthWest");
        assert_eq!(Direction::SouthEast.name(), "SouthEast");
        assert_eq!(Direction::SouthWest.name(), "SouthWest");
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::West.opposite(), Direction::East);
        assert_eq!(Direction::NorthEast.opposite(), Direction::SouthWest);
        assert_eq!(Direction::NorthWest.opposite(), Direction::SouthEast);
        assert_eq!(Direction::SouthEast.opposite(), Direction::NorthWest);
        assert_eq!(Direction::SouthWest.opposite(), Direction::NorthEast);
    }

    #[test]
    fn test_direction_right() {
        assert_eq!(Direction::North.right(), Direction::East);
        assert_eq!(Direction::South.right(), Direction::West);
        assert_eq!(Direction::East.right(), Direction::South);
        assert_eq!(Direction::West.right(), Direction::North);
        assert_eq!(Direction::NorthEast.right(), Direction::SouthEast);
        assert_eq!(Direction::NorthWest.right(), Direction::NorthEast);
        assert_eq!(Direction::SouthEast.right(), Direction::SouthWest);
        assert_eq!(Direction::SouthWest.right(), Direction::NorthWest);
    }

    #[test]
    fn test_direction_left() {
        assert_eq!(Direction::North.left(), Direction::West);
        assert_eq!(Direction::South.left(), Direction::East);
        assert_eq!(Direction::East.left(), Direction::North);
        assert_eq!(Direction::West.left(), Direction::South);
        assert_eq!(Direction::NorthEast.left(), Direction::NorthWest);
        assert_eq!(Direction::NorthWest.left(), Direction::SouthWest);
        assert_eq!(Direction::SouthEast.left(), Direction::NorthEast);
        assert_eq!(Direction::SouthWest.left(), Direction::SouthEast);
    }

    #[test]
    fn test_direction_from_str() {
        assert_eq!(Direction::from_str("North"), Some(Direction::North));
        assert_eq!(Direction::from_str("South"), Some(Direction::South));
        assert_eq!(Direction::from_str("East"), Some(Direction::East));
        assert_eq!(Direction::from_str("West"), Some(Direction::West));
        assert_eq!(Direction::from_str("NorthEast"), Some(Direction::NorthEast));
        assert_eq!(Direction::from_str("NorthWest"), Some(Direction::NorthWest));
        assert_eq!(Direction::from_str("SouthEast"), Some(Direction::SouthEast));
        assert_eq!(Direction::from_str("SouthWest"), Some(Direction::SouthWest));
        assert_eq!(Direction::from_str("Invalid"), None);
    }
}


// ===========================================================================
// Grid of Tiles

#[derive(Clone, PartialEq, Debug)]
pub enum TileType {
    Boundary,
    Mountain,
    Forest,
    Earth,
    Beach,
    Water,
    River,

    // Add more tile types as needed
}

pub trait TileTrait {
    fn new(tile_type: TileType) -> Self;
    // add accessors for contents, is_safe_zone, is_spawn_point, and elevation
    fn is_safe_zone(&self) -> bool;
    fn is_spawn_point(&self) -> bool;
    fn get_type(&self) -> &TileType;
    // Add more common functions as needed
}

pub trait EarthTileTrait {
    fn get_elevation(&self) -> Option<u32>;
    // set elevation for Earth tiles
    fn set_elevation(&mut self, elevation: u32);
    // Add more Earth-specific functions as needed
}


pub struct Tile {
    tile_type: TileType,
    contents: Vec<String>,
    is_safe_zone: bool,
    is_spawn_point: bool,
    elevation: Option<u32>,
}

impl TileTrait for Tile {
    fn new(tile_type: TileType) -> Self {
        let tile_type_clone = tile_type.clone();
        Tile {
            tile_type,
            contents: Vec::new(),
            is_safe_zone: false,
            is_spawn_point: false,
            elevation: match tile_type_clone { // Set elevation based on tile type
                TileType::Earth => Some(0), // Default elevation for Earth tiles
                _ => None, // Other tiles do not have an elevation
            },
        }
    }
    // add accessors for contents, is_safe_zone, is_spawn_point, and elevation
    fn is_safe_zone(&self) -> bool {
        self.is_safe_zone
    }

    fn is_spawn_point(&self) -> bool {
        self.is_spawn_point
    }

    fn get_type(&self) -> &TileType {
        &self.tile_type
    }

    // Implement more common functions as needed
}


impl EarthTileTrait for Tile {
    fn get_elevation(&self) -> Option<u32> {
        if let TileType::Earth = self.tile_type {
            self.elevation
        } else if let TileType::Mountain = self.tile_type {
            self.elevation
        } else {
            None
        }
    }

    // set elevation for Earth tiles
    fn set_elevation(&mut self, elevation: u32) {
        if let TileType::Earth = self.tile_type {
            self.elevation = Some(elevation);
        }
        if let TileType::Mountain = self.tile_type {
            self.elevation = Some(elevation);
        }
    }


    // Implement more Earth-specific functions as needed
}


pub struct Grid {
    tiles: Vec<Vec<Tile>>,
    image_buffer_bg: Arc<Mutex<Option<ImageBuffer<Rgb<u8>, Vec<u8>>>>>,
    image_buffer_fg: Arc<Mutex<Option<ImageBuffer<Rgb<u8>, Vec<u8>>>>>,
}

impl Grid {
    pub fn new(size: usize) -> Self {
        let mut tiles = Vec::new();
        for _ in 0..size {
            let mut row = Vec::new();
            for _ in 0..size {
                let tile = Tile::new(TileType::Earth);
                row.push(tile);
            }
            tiles.push(row);
        }
        Grid {
            tiles,
            image_buffer_bg: Arc::new(Mutex::new(None)),
            image_buffer_fg: Arc::new(Mutex::new(None)),
        }
    }


    // get neighbors of a tile
    pub fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let size = self.tiles.len();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < size as i32 && ny >= 0 && ny < size as i32 {
                    neighbors.push((nx as usize, ny as usize));
                }
            }
        }
        neighbors
    }

    // get distance between two tiles
    pub fn get_distance(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
        let dx = x1 as i32 - x2 as i32;
        let dy = y1 as i32 - y2 as i32;
        ((dx * dx + dy * dy) as f64).sqrt()
    }

    // get elevation difference between two tiles
    pub fn get_elevation_difference(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
        let elevation1 = self.tiles[x1][y1].get_elevation().unwrap_or(0) as f64;
        let elevation2 = self.tiles[x2][y2].get_elevation().unwrap_or(0) as f64;
        (elevation1 - elevation2).abs()
    }

    // get cost of moving from one tile to another
    pub fn get_cost(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
        let distance = self.get_distance(x1, y1, x2, y2);
        let elevation_difference = self.get_elevation_difference(x1, y1, x2, y2);
        distance + elevation_difference
    }


    // set tile type in the grid. If the x and y are out of bounds, do nothing
    pub fn set_tile_type(&mut self, x: usize, y: usize, tile_type: TileType) {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            self.tiles[x][y] = Tile::new(tile_type);
        }
    }

    pub fn set_tile_elevation(&mut self, x: usize, y: usize, elevation: u32) {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            self.tiles[x][y].set_elevation(elevation);
        }
    }

    pub fn increase_earth_tile_elevation(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                if let TileType::Earth = tile.get_type() {
                    let current_elevation = tile.get_elevation().unwrap_or(0);
                    tile.set_elevation(current_elevation + 1);
                }
            }
        }
    }

    pub fn set_boundary_margin(&mut self, margin: usize) {
        let size = self.tiles.len();
        for x in 0..size {
            for y in 0..size {
                if x < margin || x >= size - margin || y < margin || y >= size - margin {
                    self.set_tile_type(x, y, TileType::Boundary);
                }
            }
        }
    }


    pub fn generate_river(&mut self, initial_width: usize) {
        // let (start_x, start_y) = self.find_highest_point();
        // let (end_x, end_y) = self.find_boundary_water_tile();
    }

    pub fn generate_island(&mut self, seed: u32) {
        let perlin = Perlin::new().set_seed(seed);
        let (width, height) = (self.tiles.len(), self.tiles[0].len());
        let (center_x, center_y) = (width / 2, height / 2);

        // Generate all noise values
        let mut noise_values: Vec<f64> = Vec::new();
        for x in 0..width {
            for y in 0..height {
                let distance_to_center = (((x as i32 - center_x as i32).pow(2) + (y as i32 - center_y as i32).pow(2)) as f64).sqrt();
                let noise_value = perlin.get([x as f64 / width as f64, y as f64 / height as f64]) - (distance_to_center / width as f64);
                // Scale the noise value to the range 0 to 2
                let scaled_noise_value = noise_value * 0.5 + 0.5;
                noise_values.push(scaled_noise_value);
            }
        }

        // Find the minimum and maximum noise values
        let min_noise = *noise_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_noise = *noise_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // Set the threshold to be the average of the minimum and maximum noise values
        let threshold = (min_noise + max_noise) / 2.0;

        // Set each tile's type based on the noise value and the threshold
        self.tiles.par_iter_mut().enumerate().for_each(|(x, row)| {
            row.iter_mut().enumerate().for_each(|(y, tile)| {
                let noise_value = noise_values[x * width + y];
                if noise_value > threshold {
                    tile.tile_type = TileType::Earth;
                    // Calculate the elevation based on the distance to the center
                    let distance_to_center = (((x as i32 - center_x as i32).pow(2) + (y as i32 - center_y as i32).pow(2)) as f64).sqrt();
                    let max_distance = ((width.pow(2) + height.pow(2)) as f64).sqrt();
                    let elevation = 2.0 + (50.0 - 2.0) * (1.0 - distance_to_center / max_distance);
                    let elevation = elevation + noise_value * 10.0;
                    tile.elevation = Some(elevation as u32);

                    let mountain_noise_threshold = 0.46;
                    let mountain_elevation_threshold = 47.0;
                    if elevation > mountain_elevation_threshold
                        && noise_value > mountain_noise_threshold {
                        tile.tile_type = TileType::Mountain;
                        tile.elevation = Some(elevation as u32);
                    }
                } else {
                    tile.tile_type = TileType::Water;
                    tile.elevation = Some(0);
                }
            });
        });


        // Post-processing step to set Beach tiles
        let mut beach_tiles = Vec::new();
        let beach_width = 8; // Set the width of the beach
        for x in 0..width {
            for y in 0..height {
                if let TileType::Earth = self.tiles[x][y].tile_type {
                    for i in 0..=beach_width {
                        let neighbors = [
                            (x.saturating_sub(i), y),
                            (x.saturating_add(i), y),
                            (x, y.saturating_sub(i)),
                            (x, y.saturating_add(i)),
                        ];
                        for &(nx, ny) in &neighbors {
                            if nx < width && ny < height && matches!(self.tiles[nx][ny].tile_type, TileType::Water) {
                                beach_tiles.push((x, y));
                                break;
                            }
                        }
                    }
                }
            }
        }
        for (x, y) in beach_tiles {
            self.tiles[x][y].tile_type = TileType::Beach;
        }
    }


    fn find_highest_point(&self) -> (usize, usize) {
        let mut highest_point = (0, 0);
        let mut max_elevation = 0;
        for (x, row) in self.tiles.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                if let TileType::Earth = tile.tile_type {
                    if let Some(elevation) = tile.elevation {
                        if elevation > max_elevation {
                            max_elevation = elevation;
                            highest_point = (x, y);
                        }
                    }
                }
            }
        }
        highest_point
    }


    fn find_boundary_water_tile(&self) -> (usize, usize) {
        let (width, height) = (self.tiles.len(), self.tiles[0].len());
        for x in 0..width {
            for y in 0..height {
                if matches!(self.tiles[x][y].tile_type, TileType::Water) {
                    if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                        return (x, y);
                    }
                }
            }
        }
        (0, 0)
    }

    // image processing in grid
    // ============================================================================================


    pub fn generate_elevation_png(&self, filename: &str) {
        let size = self.tiles.len();
        //let mut img = ImageBuffer::new(size as u32, size as u32);

        let mut img = {
            let mut buffer = self.image_buffer_bg.lock().unwrap();
            match &*buffer {
                Some(buffer) => buffer.clone(),
                None => {
                    let new_buffer = ImageBuffer::new(size as u32, size as u32);
                    *buffer = Some(new_buffer.clone());
                    new_buffer
                }
            }
        };


        img.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
            let tile = &self.tiles[x as usize][y as usize];
            match tile.get_type() {
                TileType::Boundary => *pixel = Rgb([255, 255, 255]), // white

                TileType::Mountain => {
                    let elevation = tile.get_elevation().unwrap_or(0);
                    // Use a gradient of colors to represent the range of elevations
                    let green = (250 - (3 * elevation)) as u8;
                    *pixel = Rgb([0, green, 0]); // shades of green
                }

                TileType::Forest => *pixel = Rgb([34, 139, 34]), // green
                TileType::Water => *pixel = Rgb([0, 0, 255]), // blue
                TileType::River => *pixel = Rgb([0, 0, 245]), // river blue
                TileType::Beach => *pixel = Rgb([255, 255, 0]), // yellow

                TileType::Earth => {
                    let elevation = tile.get_elevation().unwrap_or(0);
                    // Use a gradient of colors to represent the range of elevations
                    let green = 255 - (3 * elevation) as u8;
                    *pixel = Rgb([0, green, 0]); // shades of green
                }
            }
        });

        img.save(filename).unwrap();
        // creates the base background image.
        *self.image_buffer_bg.lock().unwrap() = Some(img);
    }

    // this function clears the fg image buffer by loading the bg image buffer into it
    pub fn clear_image_fg(&self) {
        // Lock and clone the image_buffer_bg
        let img_bg = match &*self.image_buffer_bg.lock().unwrap() {
            Some(buffer) => buffer.clone(),
            None => return, // If the image_buffer_bg is None, return early
        };

        // Assign the cloned image_buffer_bg to image_buffer_fg
        *self.image_buffer_fg.lock().unwrap() = Some(img_bg);
    }

    // this function clones the fg image buffer and locks it
    fn clone_and_lock_fg(&self) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let mut img = match &*self.image_buffer_fg.lock().unwrap() {
            Some(buffer) => buffer.clone(),
            None => return None, // If the image buffer is None, return early
        };
        Some(img)
    }


    pub fn draw_circle(&self, center: (i32, i32), radius: i32, color: Rgb<u8>) {

        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                println!("Warning: The image buffer is None");
                return;
            }
        };

        let (width, height) = img.dimensions();
        // Check if the circle is within the image dimensions
        if (center.0 + radius > width as i32) || (center.1 + radius > height as i32) {
            println!("Warning: The circle is outside the image dimensions");
            println!("Image dimensions: {} x {}", width, height);
            println!("Circle center: {:?}, radius: {}, color: {:?}", center, radius, color);
            return;
        }

        draw_filled_circle_mut(&mut img, center, radius, color);
    }

    pub fn do_draw_circle(&self, x:i32, y:i32) {

        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                println!("Warning: The image buffer is None");
                return;
            }
        };
        let circle_color = Rgb([255, 255, 0]);
        let (width, height) = img.dimensions();
        let center = (x, y);
        let radius = 5;
        // Check if the circle is within the image dimensions
        if (center.0 + radius > width as i32) || (center.1 + radius > height as i32) {
            println!("Warning: The circle is outside the image dimensions");
            println!("Image dimensions: {} x {}", width, height);
            println!("Circle center: {:?}, radius: {}, color: {:?}", center, radius, circle_color);
            return;
        }

        draw_filled_circle_mut(&mut img, center, radius, circle_color);

        *self.image_buffer_fg.lock().unwrap() = Some(img);

    }



    // save fg image buffer to a png file
    pub fn save_image_fg(&self, filename: &str) {
        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                println!("Warning: The image buffer is None");
                return;
            }
        };

        // Save the image buffer to a file
        img.save(filename).unwrap();
    }



// end of Grid struct
}

// define our grid in a way that can be shared across threads
lazy_static! {
        static ref GRID: Mutex<Grid> = Mutex::new(Grid::new(2048));
    }

// reset the grid to its initial state
fn reset_grid() {
    let mut grid = GRID.lock().unwrap();
    *grid = Grid::new(2048);
}

#[cfg(test)]
mod tile_tests {
    use super::*;


    lazy_static! {
        static ref GRID: Mutex<Grid> = Mutex::new(Grid::new(2048));
    }

    fn reset_grid() {
        let mut grid = GRID.lock().unwrap();
        *grid = Grid::new(2048);
    }

    #[test]
    fn test_tile_creation() {
        let tile = Tile::new(TileType::Earth);
        assert_eq!(*tile.get_type(), TileType::Earth);
    }

    #[test]
    fn test_set_tile_type() {
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.set_tile_type(0, 0, TileType::Water);
        assert_eq!(*grid.tiles[0][0].get_type(), TileType::Water);
    }


    fn test_increase_earth_tile_elevation() {
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.increase_earth_tile_elevation();
        for row in &grid.tiles {
            for tile in row {
                if let TileType::Earth = tile.get_type() {
                    assert_eq!(tile.get_elevation(), Some(1));
                }
            }
        }
    }


    fn test_set_grid_tile_elevation() { //??? This fails....
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.set_tile_elevation(0, 0, 1);
        // print the elevation of the tile at 0, 0
        println!("{:?}", grid.tiles[0][0].get_elevation());
        assert_eq!(grid.tiles[0][0].get_elevation(), Some(1));
    }


    #[test]
    fn test_sequentially() {
        // test_set_grid_tile_elevation();
        test_increase_earth_tile_elevation();
    }


    #[test]
    fn test_generate_png() {
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.set_boundary_margin(5);
        grid.generate_elevation_png("elevation.png");
        assert!(Path::new("elevation.png").exists());
    }


    #[test]
    fn test_get_neighbors() {
        reset_grid(); // Reset the grid before the test
        let mut grid = GRID.lock().unwrap();
        let neighbors = grid.get_neighbors(0, 0);
        assert_eq!(neighbors.len(), 3);
    }

    #[test]
    fn test_get_distance() {
        reset_grid(); // Reset the grid before the test
        let mut grid = GRID.lock().unwrap();
        let distance = grid.get_distance(0, 0, 3, 4);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_get_elevation_difference() {
        reset_grid(); // Reset the grid before the test
        let mut grid = GRID.lock().unwrap();
        grid.set_tile_elevation(0, 0, 10);
        grid.set_tile_elevation(3, 4, 5);
        let elevation_difference = grid.get_elevation_difference(0, 0, 3, 4);
        assert_eq!(elevation_difference, 5.0);
    }

    #[test]
    fn test_get_cost() {
        reset_grid(); // Reset the grid before the test
        let mut grid = GRID.lock().unwrap();
        grid.set_tile_elevation(0, 0, 10);
        grid.set_tile_elevation(3, 4, 5);
        let cost = grid.get_cost(0, 0, 3, 4);
        assert_eq!(cost, 10.0);
    }

    #[test]
    fn test_set_boundary_margin() {
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.set_boundary_margin(5);
        for x in 0..5 {
            for y in 0..2048 {
                assert_eq!(*grid.tiles[x][y].get_type(), TileType::Boundary);
            }
        }
    }

    #[test]
    fn test_find_highest_point() {
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.set_tile_elevation(0, 0, 10);
        grid.set_tile_elevation(3, 4, 5);
        let highest_point = grid.find_highest_point();
        assert_eq!(highest_point, (0, 0));
    }

    #[test]
    fn test_find_boundary_water_tile() {
        reset_grid();
        let mut grid = GRID.lock().unwrap();
        grid.set_tile_type(0, 0, TileType::Water);
        grid.set_tile_type(2047, 2047, TileType::Water);
        let boundary_water_tile = grid.find_boundary_water_tile();
        assert_eq!(boundary_water_tile, (0, 0));
    }
}

// =================================================================================================
// Items


#[derive(Clone, Debug)]
pub struct UsefulItem {
    name: String,
    use_value: u32,
    // Add more properties as needed
}

impl UsefulItem {
    pub fn new(name: String, use_value: u32) -> Self {
        UsefulItem {
            name,
            use_value,
        }
    }

    // Add getter methods to access the properties
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_use_value(&self) -> u32 {
        self.use_value
    }
}

#[derive(Clone, Debug)]
pub struct FoodItem {
    name: String,
    nutritional_value: u32,
    // Add more properties as needed
}

// derive debug for food item

impl FoodItem {
    pub fn new(name: String, nutritional_value: u32) -> Self {
        FoodItem {
            name,
            nutritional_value,
        }
    }

    // debug for food item
    pub fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }


    // Add getter methods to access the properties
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_nutritional_value(&self) -> u32 {
        self.nutritional_value
    }
}

pub trait ItemTrait {
    fn as_debug(&self) -> &dyn std::fmt::Debug;
    fn get_name(&self) -> &String;
    fn get_use_value(&self) -> u32;
    fn get_nutritional_value(&self) -> u32;
    // show item (on display)
    fn show_item(&self) {
        println!("Item: {}", self.get_name());
    }
}

impl ItemTrait for FoodItem {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_use_value(&self) -> u32 {
        0 // Food items do not have a use value
    }

    fn get_nutritional_value(&self) -> u32 {
        self.nutritional_value
    }
}

impl ItemTrait for UsefulItem {
    fn as_debug(&self) -> &dyn Debug {
        todo!()
    }

    // show item
    fn show_item(&self) {
        println!("Item: {}", self.get_name());
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_use_value(&self) -> u32 {
        self.use_value
    }

    fn get_nutritional_value(&self) -> u32 {
        0 // Useful items do not have a nutritional value
    }
}

impl std::fmt::Debug for dyn ItemTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_debug().fmt(f)
    }
}
// a bag can hold items


// derive debug for bag
#[derive(Debug)]
pub struct Bag {
    items: Vec<Box<dyn ItemTrait>>,
    size: u32,
}

impl Bag {
    pub fn new(capacity: u32) -> Self {
        Bag {
            items: Vec::new(),
            size: capacity,
        }
    }

    pub fn add_item(&mut self, item: Box<dyn ItemTrait>) {
        if self.items.len() < self.size as usize {
            self.items.push(item);
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<Box<dyn ItemTrait>> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    // Add methods to get the total use value and nutritional value of the items in the bag
    pub fn total_use_value(&self) -> u32 {
        self.items.iter().map(|item| item.get_use_value()).sum()
    }

    pub fn total_nutritional_value(&self) -> u32 {
        self.items.iter().map(|item| item.get_nutritional_value()).sum()
    }


    pub fn add_apple(&mut self) {
        let apple = Box::new(FoodItem::new("Apple".to_string(), 50));
        self.add_item(apple);
    }

    pub fn add_banana(&mut self) {
        let banana = Box::new(FoodItem::new("Banana".to_string(), 100));
        self.add_item(banana);
    }

    pub fn add_orange(&mut self) {
        let orange = Box::new(FoodItem::new("Orange".to_string(), 60));
        self.add_item(orange);
    }

    // Add more helper functions for other common food items...
}

#[cfg(test)]
mod item_tests {
    use super::*;

    #[test]
    fn test_bag_creation() {
        let bag = Bag::new(5);
        assert_eq!(bag.items.len(), 0);
    }

    #[test]
    fn test_add_item() {
        let mut bag = Bag::new(5);
        let apple = Box::new(FoodItem::new("Apple".to_string(), 50));
        bag.add_item(apple);
        assert_eq!(bag.items.len(), 1);
    }

    #[test]
    fn test_remove_item() {
        let mut bag = Bag::new(5);
        let apple = Box::new(FoodItem::new("Apple".to_string(), 50));
        bag.add_item(apple);
        let removed_item = bag.remove_item(0);
        assert_eq!(removed_item.is_some(), true);
        assert_eq!(bag.items.len(), 0);
    }

    #[test]
    fn test_total_use_value() {
        let mut bag = Bag::new(5);
        let apple = Box::new(FoodItem::new("Apple".to_string(), 50));
        let banana = Box::new(FoodItem::new("Banana".to_string(), 100));
        bag.add_item(apple);
        bag.add_item(banana);
        assert_eq!(bag.total_use_value(), 0);
    }

    #[test]
    fn test_total_nutritional_value() {
        let mut bag = Bag::new(5);
        let apple = Box::new(FoodItem::new("Apple".to_string(), 50));
        let banana = Box::new(FoodItem::new("Banana".to_string(), 100));
        bag.add_item(apple);
        bag.add_item(banana);
        assert_eq!(bag.total_nutritional_value(), 150);
    }

    #[test]
    fn test_add_apple() {
        let mut bag = Bag::new(5);
        bag.add_apple();
        assert_eq!(bag.items.len(), 1);
    }

    #[test]
    fn test_add_banana() {
        let mut bag = Bag::new(5);
        bag.add_banana();
        assert_eq!(bag.items.len(), 1);
    }

    #[test]
    fn test_add_orange() {
        let mut bag = Bag::new(5);
        bag.add_orange();
        assert_eq!(bag.items.len(), 1);
    }

    // Add more tests for other common food items...
}


// =================================================================================================
// Characters

#[derive(PartialEq, Debug)]
pub enum CharacterType {
    Human,
    Elf,
    Dwarf,
    Gnole,
    Orc,
    Troll,
    // Add more character types as needed
}

#[derive(Debug)]
pub struct Character {
    name: String,
    character_type: CharacterType,
    health: u32,
    strength: u32,
    agility: u32,
    intelligence: u32,
    x_position: usize,
    y_position: usize,
    my_bag: Bag,
}

impl Character {
    pub fn new(name: String, character_type: CharacterType, health: u32, strength: u32, agility: u32, intelligence: u32, x_position: usize, y_position: usize) -> Self {
        Character {
            name,
            character_type,
            health,
            strength,
            agility,
            intelligence,
            x_position,
            y_position,
            my_bag: Bag::new(15),
        }
    }

    // Getter method for the character's type
    pub fn get_character_type(&self) -> &CharacterType {
        &self.character_type
    }

    pub fn get_bag_mut(&mut self) -> &mut Bag {
        &mut self.my_bag
    }

    // move character in a Direction, North, South, East, West, using the direction offset
    pub fn move_character(&mut self, direction: Direction, grid: &Grid) {
        let (dx, dy) = direction.get_offset();

        // check the lower bounds of the grid
        // if self.x_position as i32 + dx < 0 || self.y_position as i32 + dy < 0 {
        //     return;
        // }
        // // check the upper bounds of the grid
        // if self.x_position + dx as usize >= grid.tiles.len() || self.y_position + dy as usize >= grid.tiles[0].len() {
        //     return;
        // }
        //
        // check if the next position tile is a boundary
        if let TileType::Boundary = grid.tiles[(self.x_position as i32 + dx) as usize][(self.y_position as i32 + dy) as usize].tile_type {
            // display boundary message
            println!("You cannot move beyond the boundary of this world");
            return;
        }

        self.x_position = (self.x_position as i32 + dx) as usize;
        self.y_position = (self.y_position as i32 + dy) as usize;
    }


    // Other methods...


    pub fn teleport_character(&mut self, x: usize, y: usize) {
        self.x_position = x;
        self.y_position = y;
    }
}

// a character can be a player or a computer controlled character
pub struct Player {
    character: Character,
    level: u32,
    experience: u32,
}

impl Player {
    pub fn new(name: String, character_type: CharacterType, health: u32, strength: u32, agility: u32, intelligence: u32, x_position: usize, y_position: usize, level: u32, experience: u32) -> Self {
        Player {
            character: Character::new(name, character_type, health, strength, agility, intelligence, x_position, y_position),
            level,
            experience,
        }
    }

    // Getter methods for the player's level and experience
    pub fn get_level(&self) -> u32 {
        self.level
    }

    pub fn get_experience(&self) -> u32 {
        self.experience
    }

    // Setter methods for the player's level and experience
    pub fn set_level(&mut self, level: u32) {
        self.level = level;
    }

    pub fn set_experience(&mut self, experience: u32) {
        self.experience = experience;
    }
}

pub struct ComputerControlledCharacter {
    character: Character,
}

impl ComputerControlledCharacter {
    pub fn new(name: String, character_type: CharacterType, health: u32, strength: u32, agility: u32, intelligence: u32, x_position: usize, y_position: usize) -> Self {
        ComputerControlledCharacter {
            character: Character::new(name, character_type, health, strength, agility, intelligence, x_position, y_position),
        }
    }
}

pub struct CharacterManager {
    characters: Vec<Character>,
}

impl CharacterManager {
    pub fn new() -> Self {
        CharacterManager {
            characters: Vec::new(),
        }
    }

    pub fn add_character(&mut self, character: Character) {
        self.characters.push(character);
    }

    pub fn remove_character(&mut self, index: usize) -> Option<Character> {
        if index < self.characters.len() {
            Some(self.characters.remove(index))
        } else {
            None
        }
    }

    // Method to get a mutable character reference by its index
    pub fn get_character_mut(&mut self, index: usize) -> Option<&mut Character> {
        self.characters.get_mut(index)
    }


    // Method to get a character by name
    pub fn get_character_by_name(&self, name: &str) -> Option<&Character> {
        self.characters.iter().find(|character| character.name == name)
    }


    // Method to get the total number of characters
    pub fn count_characters(&self) -> usize {
        self.characters.len()
    }

    // return a list of characters at x,y on the grid
    pub fn get_characters_at_position(&self, x: usize, y: usize) -> Vec<&Character> {
        self.characters.iter().filter(|character| character.x_position == x && character.y_position == y).collect()
    }

    // is there a character at x,y on the grid
    pub fn is_character_at_position(&self, x: usize, y: usize) -> bool {
        self.characters.iter().any(|character| character.x_position == x && character.y_position == y)
    }


    // list characters in the character manager
    pub fn list_characters(&self) {
        // Display the list of characters
        for character in &self.characters {
            println!("{:?}", character);
        }
    }


    // Add more methods to perform operations on the characters...

    // convenience method to add a player to the character manager
    pub fn add_player(&mut self, player: Player) {
        self.add_character(player.character);
    }
}

pub struct NPC {
    character: Character,

}

impl NPC {
    pub fn new(name: String, character_type: CharacterType, health: u32, strength: u32, agility: u32, intelligence: u32, x_position: usize, y_position: usize, dialogue: String) -> Self {
        NPC {
            character: Character::new(name, character_type, health, strength, agility, intelligence, x_position, y_position),

        }
    }
}

#[cfg(test)]
mod character_tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        assert_eq!(*character.get_character_type(), CharacterType::Human);
    }

    #[test]
    fn test_teleport_character() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character.teleport_character(5, 5);
        assert_eq!(character.x_position, 5);
        assert_eq!(character.y_position, 5);
    }

    #[test]
    fn test_player_creation() {
        let player = Player::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, 1, 0);
        assert_eq!(player.get_level(), 1);
        assert_eq!(player.get_experience(), 0);
    }

    #[test]
    fn test_set_level() {
        let mut player = Player::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, 1, 0);
        player.set_level(2);
        assert_eq!(player.get_level(), 2);
    }

    #[test]
    fn test_set_experience() {
        let mut player = Player::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, 1, 0);
        player.set_experience(100);
        assert_eq!(player.get_experience(), 100);
    }

    #[test]
    fn test_computer_controlled_character_creation() {
        let character = ComputerControlledCharacter::new("Oscar".to_string(), CharacterType::Orc, 100, 10, 10, 10, 0, 0);
        assert_eq!(*character.character.get_character_type(), CharacterType::Orc);
    }

    #[test]
    fn test_character_manager_creation() {
        let character_manager = CharacterManager::new();
        assert_eq!(character_manager.count_characters(), 0);
    }

    #[test]
    fn test_add_character() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character_manager.add_character(character);
        assert_eq!(character_manager.count_characters(), 1);
    }

    #[test]
    fn test_remove_character() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character_manager.add_character(character);
        let removed_character = character_manager.remove_character(0);
        assert_eq!(removed_character.is_some(), true);
        assert_eq!(character_manager.count_characters(), 0);
    }

    #[test]
    fn test_get_character() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character_manager.add_character(character);
        let retrieved_character = character_manager.get_character(0);
        assert_eq!(retrieved_character.is_some(), true);
    }

    #[test]
    fn test_get_character_by_name() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character_manager.add_character(character);
        let retrieved_character = character_manager.get_character_by_name("Player");
        assert_eq!(retrieved_character.is_some(), true);
    }

    #[test]
    fn test_get_characters_at_position() {
        let mut character_manager = CharacterManager::new();
        let character1 = Character::new("Player1".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        let character2 = Character::new("Player2".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character_manager.add_character(character1);
        character_manager.add_character(character2);
        let characters_at_position = character_manager.get_characters_at_position(0, 0);
        assert_eq!(characters_at_position.len(), 2);
    }

    #[test]
    fn test_is_character_at_position() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        character_manager.add_character(character);
        let is_character_at_position = character_manager.is_character_at_position(0, 0);
        assert_eq!(is_character_at_position, true);
    }

    // test move north
    #[test]
    fn test_move_north() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        let grid = Grid::new(10);
        character.move_character(Direction::North, &grid);
        assert_eq!(character.x_position, 0);
        assert_eq!(character.y_position, 1);
    }

    // test move south
    #[test]
    fn test_move_south() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 1);
        let grid = Grid::new(10);
        character.move_character(Direction::South, &grid);
        assert_eq!(character.x_position, 0);
        assert_eq!(character.y_position, 0);
    }

    // test move east
    #[test]
    fn test_move_east() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        let grid = Grid::new(10);
        character.move_character(Direction::East, &grid);
        assert_eq!(character.x_position, 1);
        assert_eq!(character.y_position, 0);
    }

    // test move west
    #[test]
    fn test_move_west() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 1, 0);
        let grid = Grid::new(10);
        character.move_character(Direction::West, &grid);
        assert_eq!(character.x_position, 0);
        assert_eq!(character.y_position, 0);
    }

    // test move beyond boundary
    #[test]
    fn test_move_beyond_boundary() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        let grid = Grid::new(10);
        character.move_character(Direction::West, &grid);
        assert_eq!(character.x_position, 0);
        assert_eq!(character.y_position, 0);
    }

    // test move to boundary
    #[test]
    fn test_move_to_boundary() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0);
        let grid = Grid::new(10);
        character.move_character(Direction::East, &grid);
        assert_eq!(character.x_position, 1);
        assert_eq!(character.y_position, 0);
    }
}


// =================================================================================================
// interactive command line interpreter

// define commands
#[derive(Debug)]
pub enum Command {
    Move(Direction),
    Teleport(usize, usize),
    AddApple,
    AddBanana,
    AddOrange,
    ListCharacters,
    ListItems,
    ShowItem,
    ShowCharacter,
    ShowMap,
    Quit,
    Help,
    Unknown,
}

// parse user input into a command
fn parse_command(input: &str) -> Command {

    // check for no value, just return press
    if input.trim().is_empty() {
        return Command::Unknown;
    }

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    match parts[0].to_lowercase().as_str() {
        "move" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                match parts[1].to_lowercase().as_str() {
                    "north" => Command::Move(Direction::North),
                    "south" => Command::Move(Direction::South),
                    "east" => Command::Move(Direction::East),
                    "west" => Command::Move(Direction::West),
                    _ => Command::Unknown,
                }
            }
        }
        "teleport" => {
            if parts.len() < 3 {
                Command::Unknown
            } else {
                let x = parts[1].parse().unwrap_or(0);
                let y = parts[2].parse().unwrap_or(0);
                Command::Teleport(x, y)
            }
        }
        "add" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                match parts[1].to_lowercase().as_str() {
                    "apple" => Command::AddApple,
                    "banana" => Command::AddBanana,
                    "orange" => Command::AddOrange,
                    _ => Command::Unknown,
                }
            }
        }
        "list" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                match parts[1].to_lowercase().as_str() {
                    "characters" => Command::ListCharacters,
                    "items" => Command::ListItems,
                    _ => Command::Unknown,
                }
            }
        }
        "show" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                match parts[1].to_lowercase().as_str() {
                    "item" => Command::ShowItem,
                    "character" => Command::ShowCharacter,
                    "map" => Command::ShowMap,
                    _ => Command::Unknown,
                }
            }
        }
        "quit" => Command::Quit,
        "help" => Command::Help,
        _ => Command::Unknown,
    }
}

// execute a command
fn execute_command(command: Command, manager: &mut CharacterManager, grid: &mut Grid) {
    let mut player: &mut Character = manager.get_character_mut(0).unwrap();
    match command {
        Command::Move(direction) => {
            player.move_character(direction, grid);
        }
        Command::Teleport(x, y) => {
            player.teleport_character(x, y);
        }
        Command::AddApple => {
            player.get_bag_mut().add_apple();
        }
        Command::AddBanana => {
            player.get_bag_mut().add_banana();
        }
        Command::AddOrange => {
            player.get_bag_mut().add_orange();
        }
        Command::ListCharacters => {
            manager.list_characters();
        }
        Command::ListItems => {
            player.my_bag.items.iter().for_each(|item| item.show_item());
        }
        Command::ShowItem => {
            if let Some(item) = player.my_bag.items.first() {
                item.show_item();
            }
        }
        Command::ShowCharacter => {
            println!("{:?}", player);
        }
        Command::ShowMap => {
            println!("generating map ...");
            grid.clear_image_fg();
            grid.do_draw_circle(player.x_position as i32, player.y_position as i32);
            grid.save_image_fg("map.png");
        }
        Command::Quit => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        Command::Help => {
            println!("Available commands:");
            println!("move <direction> - Move the player character in the specified direction (north, south, east, west)");
            println!("teleport <x> <y> - Teleport the player character to the specified position");
            println!("add <item> - Add an item to the player character's bag (apple, banana, orange)");
            println!("list characters - List all characters in the character manager");
            println!("list items - List all items in the player character's bag");
            println!("show item - Show the first item in the player character's bag");
            println!("show character - Show the player character's details");
            println!("quit - Quit the program");
            println!("help - Show available commands");
        }
        Command::Unknown => {
            println!("Unknown command. Type 'help' to see available commands.");
        }
    }
}

// command loop
fn command_loop(manager: &mut CharacterManager, grid: &mut Grid) {
    println!("Enter command: ");
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let command = parse_command(&input);
        execute_command(command, manager, grid);
        println!("Enter command: ");
    }
}


fn main() {
    println!("Processing map ... ");

    let mut grid = GRID.lock().unwrap();

    let mut manager = CharacterManager::new();
    let player = Player::new("Kevin".to_string(),
                             CharacterType::Human,
                             100,
                             10,
                             10,
                             10,
                             500,
                             500,
                             1,
                             0);

    manager.add_player(player);

    // add troll
    let troll = ComputerControlledCharacter::new("Troll".to_string(),
                                                 CharacterType::Troll,
                                                 200,
                                                 20,
                                                 5,
                                                 5,
                                                 5,
                                                 5);
    manager.add_character(troll.character);


    grid.generate_island(17);
    grid.set_boundary_margin(5);
    grid.generate_elevation_png("elevation.png");


    // start the command loop
    // display "ready"
    println!("Ready!");

    command_loop(&mut manager, &mut grid);


}
