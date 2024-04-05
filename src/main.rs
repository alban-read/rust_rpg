// Path: src/main.rs

use std::sync::mpsc;
use std::{io, thread};
use std::time::Duration;
use std::fmt;
use std::mem::needs_drop;


// this is attempt at designing a simple role playing game in Rust

// The game will have the following features:
// - A player character with attributes such as health, attack, and defense
// - A simple system where characters interact with each other by trading, attacking, or defending.
// - Randomly generated characters with different attributes
// - A simple command based user interface to interact with the game

// This version of the game will be designed using a functional programming approach, where the game state is immutable and changes are made by creating new instances of objects with updated attributes.
// Designing a game in a more functional way in Rust can be challenging due to the language's strict borrowing and mutability rules. However, it's not impossible. Here are some suggestions:
// Immutable Data: In functional programming, data is immutable. This means that instead of changing the state of an object, you create a new object with the updated state. This can be achieved in Rust using the clone method to create new instances of objects with modified attributes.
// Advanced Types: Rust has several advanced types that can be used to design your game in a more functional way. For example, Option and Result can be used for error handling instead of using exceptions. Enum can be used to create different types of game objects. Trait can be used to define common behavior for these objects.
// Use of Iterators: Iterators in Rust are lazy, meaning they have no effect until you consume them. They are a central feature of idiomatic, functional Rust code. You can use methods like map, filter, and fold to perform operations on game objects

// implement the commands
pub enum Command {
    Test,
    Idle,
    Quit,
    MoveTo(i32, i32),
    Attack,
    Defend,
    AddItem(Item),
    Me,
    See(String),

}


pub trait GameObject {
    fn update(&self, world: &World) -> Self;
    // other common methods...
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    terrain_type: TerrainType,
    x_position: i32,
    y_position: i32,
    // other tile properties...
}

#[derive(Clone, Debug, PartialEq)]
pub enum TerrainType {
    Grass,
    Water,
    Mountain,
    // Add other terrain types here...
}

// derive clone
#[derive(Clone, Debug, PartialEq)]
pub struct GameMap {
    tiles: Vec<Vec<Tile>>,
    width: i32,
    height: i32,
}

impl GameObject for GameMap {
    fn update(&self, _world: &World) -> Self {
        // Here you can add the logic to update the game map
        // For now, we'll just return the game map as is
        // display the game map's updated state
        println!("Game map updated");
        (*self).clone()
    }
    // other common methods...
}

impl GameMap {
    pub fn new(width: i32, height: i32) -> Self {
        let mut tiles = Vec::new();

        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                let tile = Tile {
                    terrain_type: TerrainType::Grass, // default terrain type
                    x_position: x,
                    y_position: y,
                };
                row.push(tile);
            }
            tiles.push(row);
        }

        GameMap {
            tiles,
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
    health: i32,
    attack: i32,
    defense: i32,
    x_position: i32,
    y_position: i32,

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
            Command::See(_) => { (*self).clone() }
        }
    }
}


impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Character {{ character_type: {:?}, name: {}, health: {}, attack: {}, defense: {}, x_position: {}, y_position: {}, bag: {:?} }}",
               self.character_type, self.name, self.health, self.attack, self.defense, self.x_position, self.y_position, self.bag)
    }
}


impl GameObject for Character {
    fn update(&self, _world: &World) -> Self {
        // display the character's updated state
        println!("{} updated", self.name);
        (*self).clone()
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
        // handle the quit command
        if command.trim().eq_ignore_ascii_case("quit") {
            return Command::Quit;
        }
        // add the "test" command
        if command.trim().eq_ignore_ascii_case("test") {
            return Command::Test;
        }
        // add the "me" command
        if command.trim().eq_ignore_ascii_case("me") {
            return Command::Me;
        }
        // add the see <name> command
        if command.trim().starts_with("see") {
            let parts: Vec<&str> = command.trim().split_whitespace().collect();
            if parts.len() == 2 {
                return Command::See(parts[1].parse().unwrap());
            }
        }
        return Command::Idle;
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
        }
        // After executing the command, update the world state
        *self = self.update();
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
        health: 100,
        attack: 10,
        defense: 5,
        x_position: 500,
        y_position: 500,
        bag: Vec::new(),
    };

    // create a new character
    let troll = Character {
        character_type: CharacterType::Troll,
        name: "TrollOne".to_string(),
        health: 50,
        attack: 5,
        defense: 2,
        x_position: 100,
        y_position: 100,
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


    world.list_characters();

    world.command_loop();
}

