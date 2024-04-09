// Path: src/main.rs

use std::sync::mpsc;
use std::{io, thread};
use std::cmp::max;
use std::time::Duration;
use std::fmt;
use noise::{NoiseFn, Perlin};
use std::mem::needs_drop;
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use rand::Rng;

// this is attempt at designing a simple role playing game in Rust

// The game will have the following features:
// - A player character with attributes such as health.
// - A simple system where characters interact with each other by trading, attacking, or defending.
// - Randomly generated characters with different attributes
// - A simple command based user interface to interact with the game

// This version of the game will be designed using a more functional programming approach,
// //where the game state is immutable and changes are made by creating new instances of objects with updated attributes.
// Designing a game in a more functional way in Rust can be challenging due to the language's strict borrowing and mutability rules.
// Immutable Data: In functional programming, data is immutable.
// This means that instead of changing the state of an object, we create a new object with the updated state.
// This can be achieved in Rust using the clone method to create new instances of objects with modified attributes.
// Advanced Types: Rust has several advanced types that can be used to design your game in a more functional way.
// For example, Option and Result can be used for error handling instead of using exceptions.
// Enum can be used to create different types of game objects.
// Trait can be used to define common behavior for these objects.
// Use of Iterators: Iterators in Rust are lazy, meaning they have no effect until you consume them.
// They are a central feature of idiomatic, functional Rust code.
// Use methods like map, filter, and fold to perform operations on game objects


// ===========================================================================
// Direction concepts
// a character is facing in one of the eight directions.


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

    // return direction from x,y offset
    pub fn from_offset(x: i32, y: i32) -> Option<Direction> {
        match (x, y) {
            (0, -1) => Some(Direction::North),
            (0, 1) => Some(Direction::South),
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            (1, -1) => Some(Direction::NorthEast),
            (-1, -1) => Some(Direction::NorthWest),
            (1, 1) => Some(Direction::SouthEast),
            (-1, 1) => Some(Direction::SouthWest),
            _ => None,
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
    pub fn turn_right(&self) -> Direction {
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
    pub fn turn_left(&self) -> Direction {
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

    pub fn from_lower_case_str(s: &str) -> Option<Direction> {
        match s {
            "north" => Some(Direction::North),
            "south" => Some(Direction::South),
            "east" => Some(Direction::East),
            "west" => Some(Direction::West),
            "northeast" => Some(Direction::NorthEast),
            "northwest" => Some(Direction::NorthWest),
            "southeast" => Some(Direction::SouthEast),
            "southwest" => Some(Direction::SouthWest),
            _ => None,
        }
    }


    // random direction
    pub fn random_direction() -> Direction {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..8) {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            3 => Direction::West,
            4 => Direction::NorthEast,
            5 => Direction::NorthWest,
            6 => Direction::SouthEast,
            _ => Direction::SouthWest,
        }
    }
}


// implement the commands
pub enum Command {
    Test,
    Idle,
    Quit,
    MeLook,
    Look(String),
    Move,
    MoveTo(i32, i32),
    Attack,
    Defend,
    AddItem(Item),
    Me,
    See(String),

}


// the main Perlin noise generator
static PERLIN: Lazy<Perlin> = Lazy::new(|| {
    Perlin::new(7243)
});


// things are drawn on the map in different colours
// colours by name
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const SILVER: Self = Self { r: 192, g: 192, b: 192 };
    pub const GRAY: Self = Self { r: 128, g: 128, b: 128 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const MAROON: Self = Self { r: 128, g: 0, b: 0 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0 };
    pub const OLIVE: Self = Self { r: 128, g: 128, b: 0 };
    pub const LIME: Self = Self { r: 0, g: 255, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 128, b: 0 };
    pub const AQUA: Self = Self { r: 0, g: 255, b: 255 };
    pub const TEAL: Self = Self { r: 0, g: 128, b: 128 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const NAVY: Self = Self { r: 0, g: 0, b: 128 };
    pub const FUCHSIA: Self = Self { r: 255, g: 0, b: 255 };
    pub const PURPLE: Self = Self { r: 128, g: 0, b: 128 };
}

impl Color {
    pub fn to_rgb(&self) -> image::Rgb<u8> {
        image::Rgb([self.r, self.g, self.b])
    }
    pub fn lighter(&self) -> Self {
        Self {
            r: self.r.saturating_add(64),
            g: self.g.saturating_add(64),
            b: self.b.saturating_add(64),
        }
    }

    pub fn darker(&self) -> Self {
        Self {
            r: self.r.saturating_sub(64),
            g: self.g.saturating_sub(64),
            b: self.b.saturating_sub(64),
        }
    }
}


pub trait GameObject {
    fn update(&self, world: &World) -> Self;
    // return tile at x,y by creating new tile
    fn get_tile(&self, x: i32, y: i32) -> Tile;
    // other common methods...
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    terrain_type: TerrainType,
    x_position: i32,
    y_position: i32,
    elevation: i32,
    // other tile properties...
}

// implement tile
impl Tile {
    // tile is generated by an algorithm, based on its position in the world.
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        let mut is_beach = false;

        // if x and y are off the map, return limbo
        if x < 0 || y < 0 || x >= width || y >= height {
            return Tile {
                terrain_type: TerrainType::Limbo,
                x_position: x,
                y_position: y,
                elevation: 0,
            };
        }

        // if the tile is within 4 tiles of the edge of the map, return boundary
        if x < 4 || y < 4 || x >= width - 8 || y >= height - 8 {
            return Tile {
                terrain_type: TerrainType::Boundary,
                x_position: x,
                y_position: y,
                elevation: 0,
            };
        }


        let mut calculate_biased_elevation = |x: f64, y: f64, neighbors: &[(f64, f64)], map_width: f64, map_height: f64| -> f64 {
            let elevation = Self::calc_elevation(width, height, x, y, map_width, map_height);
            let mut sum_elevation = elevation;
            let mut count = 1.0;
            for (neighbor_x, neighbor_y) in neighbors {
                let neighbor_elevation = Self::calc_elevation(width, height, *neighbor_x, *neighbor_y, map_width, map_height);
                sum_elevation += neighbor_elevation;
                count += 1.0;
                // print elevation and neighbor elevation
                // println!("Elevation: {}, Neighbor Elevation: {}", elevation, neighbor_elevation);
                if elevation > 0.0 && neighbor_elevation < 1.0 {
                    is_beach = true;
                }
            }
            let average_elevation = sum_elevation / count;
            average_elevation
        };


        let mut calculate_elevation = |x: f64, y: f64, width: i32, height: i32| -> i32 {
            let neighbors = [(x - 1.0, y), (x + 1.0, y), (x, y - 1.0), (x, y + 1.0)]; // only left, right, up, down neighbors
            let elevation = calculate_biased_elevation(x as f64, y as f64, &neighbors, width as f64, height as f64);
            elevation as i32
        };

        let elevation = calculate_elevation(x.into(), y.into(), width, height);
        let terrain_type = if elevation < 1 {
            TerrainType::Water
        } else if elevation >= 1 && is_beach {
            //print!("Beach ");
            TerrainType::Beach
        } else if elevation < 100 {
            TerrainType::Grass
        } else {
            TerrainType::Mountain
        };

        Tile {
            terrain_type,
            x_position: x,
            y_position: y,
            elevation,
        }
    }

    // private static method to calculate elevation
    fn calc_elevation(width: i32, height: i32, x: f64, y: f64, map_width: f64, map_height: f64) -> f64 {
        let width_f64 = width as f64;
        let height_f64 = height as f64;
        let center_x = width_f64 / 2.0;
        let center_y = height_f64 / 2.0;
        let margin = 600.0;
        let dx = x - center_x;
        let dy = y - center_y;
        let distance_to_center = (dx * dx + dy * dy).sqrt();
        let max_distance = (width_f64 * width_f64 + height_f64 * height_f64).sqrt() - margin;
        let normalized_distance = distance_to_center / max_distance;
        let scale_factor = 60.0 * (1.0 - normalized_distance);
        let noise_value = (&*PERLIN).get([x / map_width, y / map_height]);
        let normalized_noise_value = (noise_value + 1.0) / 2.0; // Normalize to 0-1
        let scaled_noise_value = normalized_noise_value * scale_factor;
        scaled_noise_value - 20.0
    }


    pub fn execute_command(&self, command: Command) -> Self {
        match command {
            Command::MoveTo(x, y) => {
                println!("Move to ({}, {})", x, y);
                // Here you can add the logic to update the tile's position
                Tile {
                    x_position: x,
                    y_position: y,
                    ..(*self).clone()
                }
            }
            Command::Idle => { (*self).clone() }
            Command::Quit => { (*self).clone() }
            _ => { (*self).clone() }
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum TerrainType {
    Limbo,
    Boundary,
    Earth,
    Grass,
    Beach,
    Water,
    Mountain,
    // Add other terrain types here...
}

// derive clone
#[derive(Clone, Debug, PartialEq)]
pub struct GameMap {
    width: i32,
    height: i32,
}

impl GameObject for GameMap {
    fn update(&self, _world: &World) -> Self {
        // Here you can add the logic to update the game map
        // For now, we'll just return the game map as is
        println!("Game map updated");
        (*self).clone()
    }

    // return tile at x,y by creating new tile
    fn get_tile(&self, x: i32, y: i32) -> Tile {
        Tile::new(x, y, self.width, self.height)
    }


    // other common methods...
}

impl GameMap {
    pub fn new(width: i32, height: i32) -> Self {
        GameMap {
            width,
            height,
        }
    }

    // game map can execute commands
    pub fn execute_command(&self, command: Command) -> Self {
        match command {
            Command::MoveTo(x, y) => {
                println!("Move to ({}, {})", x, y);
                // Here you can add the logic to update the game map
                // For now, we'll just return the game map as is
                (*self).clone()
            }
            Command::Idle => { (*self).clone() }
            Command::Quit => { (*self).clone() }
            _ => { (*self).clone() }
        }
    }


    fn generate_map_image(&self, filename: &str) {
        let width = self.width as u32;
        let height = self.height as u32;
        let mut img = ImageBuffer::<Rgb<u8>, _>::new(width, height);

        img.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
            let tile = self.get_tile(x as i32, y as i32);
            *pixel = match tile.terrain_type {
                TerrainType::Water => Color::BLUE.to_rgb(),
                TerrainType::Limbo => Color::FUCHSIA.to_rgb(),
                TerrainType::Boundary => Color::WHITE.to_rgb(),
                TerrainType::Earth | TerrainType::Grass | TerrainType::Mountain => {
                    // generate a shade of green based on the elevation
                    let elevation = tile.elevation as f32 / 100.0;
                    let color = 100 - (elevation * 255.0) as u8;
                    Rgb([0, color, 0])
                }
                TerrainType::Beach => Color::YELLOW.to_rgb(),
            };
        });
        img.save(filename).unwrap();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CharacterType {
    Player,
    Troll,
    // Add other character types here...
}

#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    character_type: CharacterType,
    name: String,
    energy: i32,
    hydration: i32,
    health: i32,
    attack: i32,
    defense: i32,
    x_position: i32,
    y_position: i32,
    facing: Direction,
    // character has a bag of items
    bag: Vec<Item>,

}

impl Character {
    pub fn execute_command(&self, command: Command) -> Self {
        match command {
            Command::MoveTo(x, y) => {
                println!("{} moves to ({}, {})", self.name, x, y);
                // Here you can add the logic to update the character's position
                Character {
                    x_position: x,
                    y_position: y,
                    ..(*self).clone()
                }
            }
            Command::Attack => {
                println!("{} attacks", self.name);
                // Here you can add the logic for the character to attack
                // For example, you might decrease the health of the attacked character
                // and return a new Character with the updated health
                // For now, we'll just return the character as is
                (*self).clone()
            }
            Command::Defend => {
                println!("{} defends", self.name);
                // Here you can add the logic for the character to defend
                // For example, you might increase the defense of the character
                // and return a new Character with the updated defense
                // For now, we'll just return the character as is
                (*self).clone()
            }
            Command::Idle => { (*self).clone() }
            Command::Quit => { (*self).clone() }
            Command::Test => { (*self).clone() }

            Command::AddItem(item) => {
                // create a new player with the item added to the bag
                let mut new_bag = self.bag.clone();
                new_bag.push(item);
                Character {
                    bag: new_bag,
                    ..(*self).clone()
                }
            } // add item
            Command::Me => {
                println!("{}'s bag: {:?}", self.name, self.bag);
                (*self).clone()
            }
            Command::See(_) => {
                (*self).clone()
            }
            Command::MeLook => {
                (*self).clone()
            }
            Command::Look(_) => {
                (*self).clone()
            }
            Command::Move => {

                // advance in the direction we are facing
                let (x, y) = (self.x_position + self.facing.get_offset().0, self.y_position + self.facing.get_offset().1);
                println!("{} moves to ({}, {})", self.name, x, y);
                // get the tile for x,y
                let tile = Tile::new(x, y, 2048, 2048);

                // if the tile is a boundary do not move
                if tile.terrain_type == TerrainType::Boundary {
                    println!("{} cannot move to ({}, {}) because it is a boundary", self.name, x, y);
                    return (*self).clone();
                }
                // if the tile is water do not move
                if tile.terrain_type == TerrainType::Water {
                    println!("{} cannot move to ({}, {}) because it is water", self.name, x, y);
                    return (*self).clone();
                }

                Character {
                    x_position: x,
                    y_position: y,
                    ..(*self).clone()
                }
            }
        }
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Character {{ character_type: {:?}, name: {}, health: {}, energy: {}, hydration: {}, x_position: {}, y_position: {}, bag: {:?} }}",
               self.character_type, self.name, self.health, self.energy, self.hydration, self.x_position, self.y_position, self.bag)
    }
}

impl GameObject for Character {
    fn update(&self, _world: &World) -> Self {
        // display the character's updated state
        println!("{} updated", self.name);
        // reduce energy and hydration
        let energy = max(0, self.energy - 1);
        let hydration = max(0, self.hydration - 1);
        // reduce health if energy or hydration is zero
        let health = if energy == 0 || hydration == 0 {
            max(0, self.health - 1)
        } else {
            self.health
        };
        // create a new character with the updated attributes
        Character {
            energy,
            hydration,
            health,
            ..(*self).clone()
        }
    }

    fn get_tile(&self, x: i32, y: i32) -> Tile {
        Tile::new(x, y, 2048, 2048)
    }
    // other common methods...
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemType {
    Quit,
    Food,
    Weapon,
    Potion,
    // Add other item types here...
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    item_type: ItemType,
    name: String,
    value: i32,
    x_position: i32,
    y_position: i32,
}

// implement item
impl Item {
    // create a new item
    pub fn new(item_type: ItemType, name: &str, value: i32, x_position: i32, y_position: i32) -> Self {
        Item {
            item_type,
            name: name.to_string(),
            value,
            x_position,
            y_position,
        }
    }
    pub fn execute_command(&self, command: Command) -> Self {
        match command {
            Command::MoveTo(x, y) => {
                println!("{} moves to ({}, {})", self.name, x, y);
                // Here you can add the logic to update the item's position
                Item {
                    x_position: x,
                    y_position: y,
                    ..(*self).clone()
                }
            }
            Command::Idle => { (*self).clone() }
            Command::Quit => { (*self).clone() }
            _ => { (*self).clone() }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Items {
    items: Vec<Item>,
}

impl GameObject for Items {
    fn update(&self, _world: &World) -> Self {
        // Here you can add the logic to update the items
        // For now, we'll just return the items as is
        // display the items' updated state
        println!("Items updated");
        (*self).clone()
    }

    fn get_tile(&self, x: i32, y: i32) -> Tile {
        Tile::new(x, y, 2048, 2048)
    }
    // other common methods...
}

//derive clone
#[derive(Clone, Debug, PartialEq)]
pub enum GameEntity {
    Character(Character),
    GameMap(GameMap),
    Items(Items),
    // other game entities...
}

impl GameObject for GameEntity {
    fn update(&self, world: &World) -> Self {
        match self {
            GameEntity::Character(character) => GameEntity::Character(character.update(world)),
            GameEntity::GameMap(game_map) => GameEntity::GameMap(game_map.update(world)),
            GameEntity::Items(items) => GameEntity::Items(items.update(world)),
            // other game entities...
        }
    }

    fn get_tile(&self, x: i32, y: i32) -> Tile {
        match self {
            GameEntity::Character(character) => character.get_tile(x, y),
            GameEntity::GameMap(game_map) => game_map.get_tile(x, y),
            GameEntity::Items(items) => items.get_tile(x, y),
            // other game entities...
        }
    }
    // other common methods...
}

// Define the world state
pub struct World {
    height: i32,
    width: i32,
    entities: Vec<GameEntity>,
    game_map: GameMap,


    // other world state...
}

// Update the world state in a functional way
impl World {
    pub fn update(&self) -> Self {
        println!("World updated");
        World {
            height: 2048,
            width: 2048,
            entities: self.entities.iter().map(|entity| entity.update(self)).collect(),
            // create new game map with updated tiles
            game_map: GameMap::new(self.width, self.height),
            // other world state...
        }
    }

    pub fn command_loop(&mut self) {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        match tx.send(input) {
                            Ok(_) => (),
                            Err(e) => println!("Failed to send input: {}", e),
                        }
                    }
                    Err(e) => println!("Failed to read line: {}", e),
                }
            }
        });

        loop {
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(input) => {
                    let commands: Vec<&str> = input.trim().split(";").collect();
                    for command in commands {
                        if command.trim().is_empty() {
                            continue;
                        }
                        let command = self.parse_command(command);
                        self.execute_command(command);
                    }
                    println!("Enter command: ");
                }
                Err(_) => { self.execute_command(Command::Idle) }
            }
        }
    }

    fn parse_command(&self, command: &str) -> Command {
        let command = command.trim();
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts.get(0) {
            Some(&"quit") => Command::Quit,
            Some(&"test") => Command::Test,
            Some(&"me") => Command::Me,
            Some(&"move") => Command::Move,
            Some(&"see") if parts.len() == 2 => Command::See(parts[1].parse().unwrap()),
            Some(&"look") if parts.len() == 2 => Command::Look(parts[1].parse().unwrap()),
            _ => Command::Idle,
        }
    }

    fn execute_command(&mut self, command: Command) {
        // execute the command
        match command {
            Command::MoveTo(x, y) => {
                // Here you can add the logic to move a character
                println!("Move to ({}, {})", x, y);
            }
            Command::Attack => {
                // Here you can add the logic for a character to attack
                println!("Attack");
            }
            Command::Defend => {
                // Here you can add the logic for a character to defend
                println!("Defend");
            }
            Command::Quit => {
                // Here you can add the logic to quit the game
                println!("Quit");
                std::process::exit(0);
            }
            Command::Idle => {
                // Here you can add the logic for the idle command
                println!("Idle");
            }
            Command::Test => {
                // Here you can add the logic for the test command
                println!("Test");
                // get player character
                let player = self.search_for_player();
                if let Some(player) = player {
                    println!("Player found: {}", player.name);
                } else {
                    println!("Player not found");
                }
            }
            Command::AddItem(_) => {}
            Command::Me => {
                // find and display the player
                self.find_and_display_player();
            }
            Command::See(name) => {
                // find and display the character by name
                self.find_and_display_character(name);
            }
            // add command "look"
            Command::Look(name) => {
                let character = self.search_for_named_character(name);
                if let Some(character) = character {
                    // get the character's position
                    let x = character.x_position;
                    let y = character.y_position;
                    println!("{:?}", character.get_tile(x, y))
                } else {
                    println!("Character not found");
                }
            }
            // add command "me look"
            Command::MeLook => {
                // look around the player
                println!("Look around player");
            }
            Command::Move => {
                let player = self.search_for_player();
                if let Some(player) = player {
                    let new_player = player.execute_command(Command::Move);
                    self.remove_named_character(player.name.clone());
                    self.add_character(new_player);
                }
            }
        }
        // After executing the command, update the world state
        *self = self.update();
    }

    fn search_for_player(&mut self) -> Option<Character> {
        let player = self.entities.iter().find_map(|entity| {
            match entity {
                GameEntity::Character(character) => {
                    if character.character_type == CharacterType::Player {
                        Some(character.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        });
        player
    }

    pub fn remove_entity(&mut self, entity_to_remove: &GameEntity) {
        self.entities.retain(|entity| entity != entity_to_remove);
    }

    pub fn find_and_remove_character(&mut self, character_name: &str) -> Option<Character> {
        let index = self.entities.iter().position(|entity| {
            match entity {
                GameEntity::Character(character) => character.name == character_name,
                _ => false,
            }
        });

        if let Some(index) = index {
            if let GameEntity::Character(character) = self.entities.remove(index) {
                Some(character)
            } else {
                None
            }
        } else {
            None
        }
    }


    pub fn add_character(&mut self, character: Character) {
        let entity = GameEntity::Character(character);
        self.entities.push(entity);
    }
    // add item to items
    pub fn add_item(&mut self, item: Item) {
        let entity = GameEntity::Items(Items { items: vec![item] });
        self.entities.push(entity);
    }


    pub fn add_item_to_characters_bag(&mut self, item: Item, player_name: &str) {
        let character = self.find_and_remove_character(player_name);
        if let Some(character) = character {
            let entity = GameEntity::Character(character.execute_command(Command::AddItem(item.clone())));
            self.entities.push(entity);
        }
    }

    // find and display the player
    pub fn find_and_display_player(&self) {
        let player = self.entities.iter().find_map(|entity| {
            match entity {
                GameEntity::Character(character) => {
                    if character.character_type == CharacterType::Player {
                        Some(character.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        });

        if let Some(player) = player {
            println!("Player found: {}", player);
        } else {
            println!("Player not found");
        }
    }

    // find and display character by name
    pub fn find_and_display_character(&self, name: String) {
        let character = self.search_for_named_character(name);
        if let Some(character) = character {
            println!("Character found: {}", character);
        } else {
            println!("Character not found");
        }
    }

    fn search_for_named_character(&self, name: String) -> Option<Character> {
        let character = self.entities.iter().find_map(|entity| {
            match entity {
                GameEntity::Character(character) => {
                    if character.name == name {
                        Some(character.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        });
        character
    }

    // remove character by name
    pub fn remove_named_character(&mut self, name: String) {
        self.entities.retain(|entity| {
            match entity {
                GameEntity::Character(character) => character.name != name,
                _ => true,
            }
        });
    }


    // list the entities that are characters
    pub fn list_characters(&self) {
        for entity in &self.entities {
            match entity {
                GameEntity::Character(character) => {
                    println!("{}", character);
                }
                _ => (),
            }
        }
    }
}

fn main() {
    let mut world = World {
        height: 2048,
        width: 2048,
        entities: Vec::new(),
        game_map: GameMap::new(2048, 2048),
        // other world state...
    };

    let player = Character {
        character_type: CharacterType::Player,
        name: "PlayerOne".to_string(),
        energy: 1000,
        hydration: 1000,
        health: 100,
        attack: 10,
        defense: 5,
        x_position: 700,
        y_position: 500,
        facing: Direction::North,
        bag: Vec::new(),
    };

    // create a new character
    let troll = Character {
        character_type: CharacterType::Troll,
        name: "TrollOne".to_string(),
        energy: 1000,
        hydration: 1000,
        health: 150,
        attack: 5,
        defense: 2,
        x_position: 800,
        y_position: 900,
        facing: Direction::East,
        bag: Vec::new(),
    };

    world.add_character(player);
    world.add_character(troll);


    {
        let item = Item {
            item_type: ItemType::Food,
            name: "Apple".to_string(),
            value: 10,
            x_position: 500,
            y_position: 500,
        };
        world.add_item_to_characters_bag(item, "PlayerOne");
    }
    {
        let item = Item {
            item_type: ItemType::Food,
            name: "Apple".to_string(),
            value: 10,
            x_position: 500,
            y_position: 500,
        };
        world.add_item(item);
    }


    world.game_map.generate_map_image("elevation_map.png");

    world.list_characters();

    world.command_loop();
}

