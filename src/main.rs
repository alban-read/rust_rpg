use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use lazy_static::lazy_static;
use std::path::Path;
use std::sync::Mutex;
use noise::{NoiseFn, Perlin, Seedable};


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
        Grid { tiles }
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
        let (start_x, start_y) = self.find_highest_point();
        let (end_x, end_y) = self.find_boundary_water_tile();
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


    pub fn generate_elevation_png(&self, filename: &str) {
        let size = self.tiles.len();
        let mut img = ImageBuffer::new(size as u32, size as u32);

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
    }

// Add methods for manipulating and accessing the grid as needed
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


#[derive(Clone)]
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

#[derive(Clone)]
pub struct FoodItem {
    name: String,
    nutritional_value: u32,
    // Add more properties as needed
}

impl FoodItem {
    pub fn new(name: String, nutritional_value: u32) -> Self {
        FoodItem {
            name,
            nutritional_value,
        }
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
    fn get_name(&self) -> &String;
    fn get_use_value(&self) -> u32;
    fn get_nutritional_value(&self) -> u32;
}

impl ItemTrait for FoodItem {
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


// a bag can hold items

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

pub enum CharacterType {
    Human,
    Elf,
    Dwarf,
    Gnole,
    Orc,
    Troll,
    // Add more character types as needed
}

pub struct Character {
    name: String,
    character_type: CharacterType,
    health: u32,
    strength: u32,
    agility: u32,
    intelligence: u32,
    x_position: usize,
    y_position: usize,
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
        }
    }

    // Getter method for the character's type
    pub fn get_character_type(&self) -> &CharacterType {
        &self.character_type
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

    // Method to get a character by its index
    pub fn get_character(&self, index: usize) -> Option<&Character> {
        self.characters.get(index)
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


    // Add more methods to perform operations on the characters...
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


fn main() {
    println!("Hello, world!");
    let mut grid = GRID.lock().unwrap();
    grid.generate_island(17);
    grid.set_boundary_margin(5);
    grid.generate_elevation_png("elevation.png");
}
