// (c) Alban 2024
// this is attempt 1.
// I became unhappy with the way mutable state has to be passed down into functions in other objects and they
// all have to mutate each other.

use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;
use std::any::Any;
use std::sync::mpsc;
use std::io::{self, BufRead};
use std::time::Duration;
use std::thread;
use image::{ImageBuffer, Rgb};
use imageproc::drawing::{draw_filled_circle, draw_filled_rect, draw_hollow_rect, draw_antialiased_line_segment, draw_polygon_mut, draw_filled_circle_mut, draw_cross};
use imageproc::pixelops::interpolate;
use rayon::prelude::*;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use imageproc::point::Point;
use imageproc::rect::Rect;
use noise::{NoiseFn, Perlin, Seedable};
use rand::prelude::{IteratorRandom, ThreadRng};
use rand::Rng;
use rand::seq::SliceRandom;


// implement macro for unless( condition) { block }
macro_rules! unless {
    ($cond:expr, $block:block) => {
        if !$cond $block
    };
}

// implement dotimes(n) { repeat block n times }
macro_rules! dotimes {
    ($n:expr, $block:block) => {
        for _ in 0..$n $block
    };
}

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    }
}


// =================================================================================================
// useful utils

pub fn proper_name(name: String) -> String {
    let binding = name.to_lowercase();
    let mut chars = binding.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}


// predefined colours named Color_XXX
pub const COLOR_RED: Rgb<u8> = Rgb([255, 0, 0]);
pub const COLOR_GREEN: Rgb<u8> = Rgb([0, 255, 0]);
pub const COLOR_BLUE: Rgb<u8> = Rgb([0, 0, 255]);
pub const COLOR_YELLOW: Rgb<u8> = Rgb([255, 255, 0]);
pub const COLOR_WHITE: Rgb<u8> = Rgb([255, 255, 255]);
pub const COLOR_BLACK: Rgb<u8> = Rgb([0, 0, 0]);
pub const COLOR_GRAY: Rgb<u8> = Rgb([128, 128, 128]);
pub const COLOR_LIGHT_GRAY: Rgb<u8> = Rgb([192, 192, 192]);
pub const COLOR_DARK_GRAY: Rgb<u8> = Rgb([64, 64, 64]);
pub const COLOR_ORANGE: Rgb<u8> = Rgb([255, 165, 0]);
pub const COLOR_PURPLE: Rgb<u8> = Rgb([128, 0, 128]);
pub const COLOR_CYAN: Rgb<u8> = Rgb([0, 255, 255]);
pub const COLOR_PINK: Rgb<u8> = Rgb([255, 192, 203]);
pub const COLOR_BROWN: Rgb<u8> = Rgb([165, 42, 42]);
// magenta
pub const COLOR_MAGENTA: Rgb<u8> = Rgb([255, 0, 255]);
// lime
pub const COLOR_LIME: Rgb<u8> = Rgb([0, 255, 0]);
// olive
pub const COLOR_OLIVE: Rgb<u8> = Rgb([128, 128, 0]);
// maroon
pub const COLOR_MAROON: Rgb<u8> = Rgb([128, 0, 0]);
// navy
pub const COLOR_NAVY: Rgb<u8> = Rgb([0, 0, 128]);
// teal
pub const COLOR_TEAL: Rgb<u8> = Rgb([0, 128, 128]);
// silver
pub const COLOR_SILVER: Rgb<u8> = Rgb([192, 192, 192]);
// gold
pub const COLOR_GOLD: Rgb<u8> = Rgb([255, 215, 0]);
// indigo
pub const COLOR_INDIGO: Rgb<u8> = Rgb([75, 0, 130]);
// violet
pub const COLOR_VIOLET: Rgb<u8> = Rgb([238, 130, 238]);
// turquoise
pub const COLOR_TURQUOISE: Rgb<u8> = Rgb([64, 224, 208]);
// sky blue
pub const COLOR_SKY_BLUE: Rgb<u8> = Rgb([135, 206, 235]);
// light blue
pub const COLOR_LIGHT_BLUE: Rgb<u8> = Rgb([173, 216, 230]);
// dark blue
pub const COLOR_DARK_BLUE: Rgb<u8> = Rgb([0, 0, 139]);
// light green
pub const COLOR_LIGHT_GREEN: Rgb<u8> = Rgb([144, 238, 144]);
// dark green
pub const COLOR_DARK_GREEN: Rgb<u8> = Rgb([0, 100, 0]);
// light yellow
pub const COLOR_LIGHT_YELLOW: Rgb<u8> = Rgb([255, 255, 224]);
// dark yellow
pub const COLOR_DARK_YELLOW: Rgb<u8> = Rgb([189, 183, 107]);
// light orange
pub const COLOR_LIGHT_ORANGE: Rgb<u8> = Rgb([255, 160, 122]);
// dark orange
pub const COLOR_DARK_ORANGE: Rgb<u8> = Rgb([255, 140, 0]);
// light red
pub const COLOR_LIGHT_RED: Rgb<u8> = Rgb([255, 99, 71]);
// dark red
pub const COLOR_DARK_RED: Rgb<u8> = Rgb([139, 0, 0]);
// light purple
pub const COLOR_LIGHT_PURPLE: Rgb<u8> = Rgb([221, 160, 221]);
// dark purple
pub const COLOR_DARK_PURPLE: Rgb<u8> = Rgb([128, 0, 128]);
// light cyan
pub const COLOR_LIGHT_CYAN: Rgb<u8> = Rgb([224, 255, 255]);
// dark cyan
pub const COLOR_DARK_CYAN: Rgb<u8> = Rgb([0, 139, 139]);
// light pink
pub const COLOR_LIGHT_PINK: Rgb<u8> = Rgb([255, 182, 193]);
// dark pink
pub const COLOR_DARK_PINK: Rgb<u8> = Rgb([199, 21, 133]);
// light brown
pub const COLOR_LIGHT_BROWN: Rgb<u8> = Rgb([205, 133, 63]);
// dark brown
pub const COLOR_DARK_BROWN: Rgb<u8> = Rgb([139, 69, 19]);
// light magenta
pub const COLOR_LIGHT_MAGENTA: Rgb<u8> = Rgb([255, 119, 255]);
// dark magenta
pub const COLOR_DARK_MAGENTA: Rgb<u8> = Rgb([139, 0, 139]);
// light lime
pub const COLOR_LIGHT_LIME: Rgb<u8> = Rgb([204, 255, 204]);
// dark lime
pub const COLOR_DARK_LIME: Rgb<u8> = Rgb([0, 204, 0]);
// light olive
pub const COLOR_LIGHT_OLIVE: Rgb<u8> = Rgb([204, 204, 0]);
// dark olive
pub const COLOR_DARK_OLIVE: Rgb<u8> = Rgb([102, 102, 0]);
// light maroon
pub const COLOR_LIGHT_MAROON: Rgb<u8> = Rgb([204, 0, 0]);
// dark maroon
pub const COLOR_DARK_MAROON: Rgb<u8> = Rgb([102, 0, 0]);
// light navy
pub const COLOR_LIGHT_NAVY: Rgb<u8> = Rgb([0, 0, 204]);
// dark navy
pub const COLOR_DARK_NAVY: Rgb<u8> = Rgb([0, 0, 102]);
// light teal
pub const COLOR_LIGHT_TEAL: Rgb<u8> = Rgb([0, 204, 204]);
// dark teal

pub const COLOR_DARK_TEAL: Rgb<u8> = Rgb([0, 102, 102]);
// light silver
pub const COLOR_LIGHT_SILVER: Rgb<u8> = Rgb([224, 224, 224]);
// dark silver
pub const COLOR_DARK_SILVER: Rgb<u8> = Rgb([96, 96, 96]);
// light gold
pub const COLOR_LIGHT_GOLD: Rgb<u8> = Rgb([255, 236, 139]);
// dark gold
pub const COLOR_DARK_GOLD: Rgb<u8> = Rgb([184, 134, 11]);
// light indigo

pub const COLOR_LIGHT_INDIGO: Rgb<u8> = Rgb([111, 0, 255]);
// dark indigo
pub const COLOR_DARK_INDIGO: Rgb<u8> = Rgb([54, 0, 139]);
// light violet
pub const COLOR_LIGHT_VIOLET: Rgb<u8> = Rgb([238, 130, 238]);
// dark violet
pub const COLOR_DARK_VIOLET: Rgb<u8> = Rgb([148, 0, 211]);
// light turquoise
pub const COLOR_LIGHT_TURQUOISE: Rgb<u8> = Rgb([175, 238, 238]);
// dark turquoise
pub const COLOR_DARK_TURQUOISE: Rgb<u8> = Rgb([0, 206, 209]);
// light sky blue
pub const COLOR_LIGHT_SKY_BLUE: Rgb<u8> = Rgb([135, 206, 250]);

// that will do.


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

    // isfood = true
    pub fn is_food(&self) -> bool {
        true
    }


}

pub trait ItemTrait: Sync + Send {
    fn as_debug(&self) -> &dyn std::fmt::Debug;
    fn as_any(&self) -> &dyn Any;
    fn get_name(&self) -> &String;
    fn get_use_value(&self) -> u32;
    fn get_nutritional_value(&self) -> u32;
    // show item (on display)
    fn show_item(&self) {
        println!("Item: {}", self.get_name());
    }
    fn clone_box(&self) -> Box<dyn ItemTrait>;

    fn is_food(&self) -> bool;

    fn get_energy(&self) -> u32;
}


impl ItemTrait for FoodItem {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }
    fn as_any(&self) -> &dyn Any { self }
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_use_value(&self) -> u32 {
        0 // Food items do not have a use value
    }

    fn get_nutritional_value(&self) -> u32 {
        self.nutritional_value
    }

    fn clone_box(&self) -> Box<dyn ItemTrait> {
        Box::new(self.clone())
    }

    fn is_food(&self) -> bool {
        true
    }

    fn get_energy(&self) -> u32 {
        self.nutritional_value
    }


}

impl Clone for Box<dyn ItemTrait> {
    fn clone(&self) -> Box<dyn ItemTrait> {
        self.clone_box()
    }
}

impl ItemTrait for UsefulItem {
    fn as_debug(&self) -> &dyn Debug {
        todo!()
    }
    fn as_any(&self) -> &dyn Any { self }
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_use_value(&self) -> u32 {
        self.use_value
    }

    fn get_nutritional_value(&self) -> u32 {
        0 // Useful items do not have a nutritional value
    }

    // show item
    fn show_item(&self) {
        println!("Item: {}", self.get_name());
    }

    fn clone_box(&self) -> Box<dyn ItemTrait> {
        Box::new(self.clone())
    }

    fn is_food(&self) -> bool {
        false
    }

    fn get_energy(&self) -> u32 {
        0
    }
}

impl std::fmt::Debug for dyn ItemTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_debug().fmt(f)
    }
}


// a bag can hold items, a character has a bag.
// derive debug for bag
#[derive(Debug, Clone)]
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

    pub fn add_item(&mut self, item: Box<FoodItem>) {
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

    // has_food true if there is food in the bag
    pub fn has_food(&self) -> bool {
        self.items.iter().any(|item| item.is_food())
    }

    // has_useful true if there is useful item in the bag
    pub fn has_useful(&self) -> bool {
        self.items.iter().any(|item| !item.is_food())
    }

    // remove and return the first food item in the bag
    pub fn remove_first_food(&mut self) -> Option<Box<dyn ItemTrait>> {
        let index = self.items.iter().position(|item| item.is_food());
        if let Some(index) = index {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    // remove least-nutritional value food item
    pub fn remove_least_nutritious_food(&mut self) -> Option<Box<dyn ItemTrait>> {
        let index = self.items.iter().enumerate().min_by_key(|(_, item)| item.get_nutritional_value());
        if let Some((index, _)) = index {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    // is the bag full, it has len >= capacity
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.size as usize
    }


    // Add more helper functions for other common food items...
}

static FOOD_ITEMS: Lazy<Vec<Box<dyn ItemTrait>>> = Lazy::new(|| {
    vec![
        Box::new(FoodItem::new("Apple".to_string(), 10)),
        Box::new(FoodItem::new("Banana".to_string(), 15)),
        Box::new(FoodItem::new("Orange".to_string(), 20)),
        Box::new(FoodItem::new("Grapes".to_string(), 25)),
        Box::new(FoodItem::new("Strawberry".to_string(), 30)),
        Box::new(FoodItem::new("Blueberry".to_string(), 35)),
        Box::new(FoodItem::new("Raspberry".to_string(), 40)),
        Box::new(FoodItem::new("Blackberry".to_string(), 45)),
        Box::new(FoodItem::new("Pineapple".to_string(), 50)),
        Box::new(FoodItem::new("Watermelon".to_string(), 55)),
        Box::new(FoodItem::new("Kiwi".to_string(), 60)),
        Box::new(FoodItem::new("Mango".to_string(), 65)),
        Box::new(FoodItem::new("Peach".to_string(), 70)),
        Box::new(FoodItem::new("Plum".to_string(), 75)),
        Box::new(FoodItem::new("Cherry".to_string(), 80)),
        Box::new(FoodItem::new("Pear".to_string(), 85)),
        Box::new(FoodItem::new("Pomegranate".to_string(), 90)),
        Box::new(FoodItem::new("Apricot".to_string(), 95)),
        Box::new(FoodItem::new("Cantaloupe".to_string(), 100)),
        Box::new(FoodItem::new("Honeydew".to_string(), 105)),
        Box::new(FoodItem::new("Lemon".to_string(), 110)),
        Box::new(FoodItem::new("Lime".to_string(), 115)),
        Box::new(FoodItem::new("Coconut".to_string(), 120)),
        Box::new(FoodItem::new("Grapefruit".to_string(), 125)),
        Box::new(FoodItem::new("Tangerine".to_string(), 130)),
        Box::new(FoodItem::new("Nectarine".to_string(), 135)),
        Box::new(FoodItem::new("Persimmon".to_string(), 140)),
        Box::new(FoodItem::new("Starfruit".to_string(), 145)),
        Box::new(FoodItem::new("Passionfruit".to_string(), 150)),
        Box::new(FoodItem::new("Dragonfruit".to_string(), 155)),
        Box::new(FoodItem::new("Guava".to_string(), 160)),
        Box::new(FoodItem::new("Papaya".to_string(), 165)),
        Box::new(FoodItem::new("Lychee".to_string(), 170)),
        Box::new(FoodItem::new("Jackfruit".to_string(), 175)),
        Box::new(FoodItem::new("Durian".to_string(), 180)),
        Box::new(FoodItem::new("Mangosteen".to_string(), 185)),
        Box::new(FoodItem::new("Kiwi".to_string(), 190)),
        Box::new(FoodItem::new("Pineapple".to_string(), 195)),
        Box::new(FoodItem::new("Watermelon".to_string(), 200)),
        Box::new(FoodItem::new("EnergyDrink".to_string(), 200)),
    ]
});

fn get_random_food_item(rng: &mut ThreadRng) -> &Box<dyn ItemTrait> {
    let random_index = rng.gen_range(0..FOOD_ITEMS.len());
    &FOOD_ITEMS[random_index]
}

static USEFUL_ITEMS: Lazy<Vec<Box<dyn ItemTrait>>> = Lazy::new(|| {
    vec![
        Box::new(UsefulItem::new("Medkit".to_string(), 50)),
        Box::new(UsefulItem::new("Axe".to_string(), 45)),
        Box::new(UsefulItem::new("Shovel".to_string(), 50)),
        Box::new(UsefulItem::new("Pickaxe".to_string(), 55)),
        Box::new(UsefulItem::new("Knife".to_string(), 60)),
        Box::new(UsefulItem::new("Sword".to_string(), 65)),
        Box::new(UsefulItem::new("Shield".to_string(), 70)),
        Box::new(UsefulItem::new("Bow".to_string(), 75)),
        Box::new(UsefulItem::new("Crossbow".to_string(), 80)),
        Box::new(UsefulItem::new("Arrows".to_string(), 85)),
        Box::new(UsefulItem::new("Bolts".to_string(), 90)),
        Box::new(UsefulItem::new("Quiver".to_string(), 95)),
        // Add more useful items as needed
    ]
});

fn get_random_useful_item(rng: &mut ThreadRng) -> &Box<dyn ItemTrait> {
    let random_index = rng.gen_range(0..USEFUL_ITEMS.len());
    &USEFUL_ITEMS[random_index] // Return a reference to the random useful item
}


// =================================================================================================
// map items


// MapItem struct used to store items on the map

pub struct MapItem {
    item: Box<dyn ItemTrait>,
    x: usize,
    y: usize,
}

impl Clone for MapItem {
    fn clone(&self) -> Self {
        MapItem {
            item: self.item.clone_box(),
            x: self.x,
            y: self.y,
        }
    }
}

// MapItem implementation
impl MapItem {
    pub fn new(item: Box<dyn ItemTrait>, x: usize, y: usize) -> Self {
        MapItem {
            item,
            x,
            y,
        }
    }

    pub fn get_position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn get_item(&self) -> &Box<dyn ItemTrait> {
        &self.item
    }

    // remove this item from the map at x,y


    // end of MapItem struct
}

// map item grid
pub struct MapItemGrid {
    items: Vec<Vec<Option<MapItem>>>,
}

// implement clone
impl Clone for MapItemGrid {
    fn clone(&self) -> Self {
        let mut new_items = Vec::new();
        for row in &self.items {
            let mut new_row = Vec::new();
            for item in row {
                if let Some(map_item) = item {
                    new_row.push(Some(map_item.clone()));
                } else {
                    new_row.push(None);
                }
            }
            new_items.push(new_row);
        }
        MapItemGrid {
            items: new_items,
        }
    }
}


// MapItemGrid implementation
impl MapItemGrid {
    pub fn new(size: usize) -> Self {
        let mut items = Vec::new();
        for _ in 0..size {
            let mut row = Vec::new();
            for _ in 0..size {
                row.push(None);
            }
            items.push(row);
        }
        MapItemGrid {
            items,
        }
    }

    pub fn add_item(&mut self, item: MapItem) {
        let (x, y) = item.get_position();
        self.items[x][y] = Some(item);
    }

    pub fn remove_items(&mut self, x: usize, y: usize) {
        self.items[x][y] = None;
    }

    pub fn remove_named_item_at(&mut self, x: usize, y: usize, item_name: &String) {
        if let Some(map_item) = &mut self.items[x][y] {
            if map_item.get_item().get_name() == item_name {
                self.items[x][y] = None;
            }
        }
    }


    pub fn get_item(&self, x: usize, y: usize) -> Option<&MapItem> {
        self.items[x][y].as_ref()
    }

    // add random food items to the map
    pub fn add_random_food_items(&mut self, num_items: usize, grid: &mut MutexGuard<Grid>) {
        let size = 2048; // map size
        let mut rng = rand::thread_rng();
        for _ in 0..num_items {
            // choose a random map position
            let x = rng.gen_range(0..size);
            let y = rng.gen_range(0..size);
            // check if the tile at x,y is not water

            if grid.get_tile(x, y).is_not_water() {
                let food_item = get_random_food_item(&mut rng);
                // create a new MapItem with the chosen food item and add it to the map
                let map_item = MapItem::new(food_item.clone(), x, y);
                self.add_item(map_item);
                // print tile type
                //println!("Tile type: {:?}", grid.get_tile(x, y).get_tile_type_text());

                //println!("Added food item: {} at position: ({}, {})", food_item.get_name(), x, y);
            }
        }
    }

    // add random useful items to the map
    pub fn add_random_useful_items(&mut self, num_items: usize, grid: &mut MutexGuard<Grid>) {
        let size = 2048; // map size
        let mut rng = rand::thread_rng();
        for _ in 0..num_items {
            // choose a random map position
            let x = rng.gen_range(0..size);
            let y = rng.gen_range(0..size);
            // check if the tile at x,y is not water
            if grid.get_tile(x, y).is_not_water() {
                let useful_item = get_random_useful_item(&mut rng);
                // create a new MapItem with the chosen useful item and add it to the map
                let map_item = MapItem::new(useful_item.clone(), x, y);
                self.add_item(map_item);
                // print tile type
                //println!("Tile type: {:?}", grid.get_tile(x, y).get_tile_type_text());
                // println!("Added useful item: {} at position: ({}, {})", useful_item.get_name(), x, y);
            }
        }
    }


    // get items at x,y location
    pub fn get_items_at(&self, x: usize, y: usize) -> Vec<&MapItem> {
        let mut items = Vec::new();
        if let Some(item) = &self.items[x][y] {
            items.push(item);
        }
        items
    }

    // display items at x,y location
    pub fn display_items_at(&self, x: usize, y: usize) {
        let items = self.get_items_at(x, y);
        if items.is_empty() {
            println!("No items found at position: ({}, {})", x, y);
        } else {
            println!("Items found at position: ({}, {})", x, y);
            for item in items {
                println!("Item: {}", item.get_item().get_name());
            }
        }
    }

    // check if x,y location has items
    pub fn has_items_at(&self, x: usize, y: usize) -> bool {
        self.items[x][y].is_some()
    }

    // check if x,y location has food items
    pub fn has_food_items_at(&self, x: usize, y: usize) -> bool {
        if let Some(item) = &self.items[x][y] {
            return item.get_item().is_food();
        }
        false
    }



    // end of MapItemGrid struct
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

// direction tests
#[cfg(test)]
mod direction_tests {
    use super::*;

    // test from_offset
    #[test]
    fn test_direction_from_offset() {
        assert_eq!(Direction::from_offset(0, -1), Some(Direction::North));
        assert_eq!(Direction::from_offset(0, 1), Some(Direction::South));
        assert_eq!(Direction::from_offset(1, 0), Some(Direction::East));
        assert_eq!(Direction::from_offset(-1, 0), Some(Direction::West));
        assert_eq!(Direction::from_offset(1, -1), Some(Direction::NorthEast));
        assert_eq!(Direction::from_offset(-1, -1), Some(Direction::NorthWest));
        assert_eq!(Direction::from_offset(1, 1), Some(Direction::SouthEast));
        assert_eq!(Direction::from_offset(-1, 1), Some(Direction::SouthWest));
        assert_eq!(Direction::from_offset(2, 2), None);
    }

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
        assert_eq!(Direction::North.turn_right(), Direction::East);
        assert_eq!(Direction::South.turn_right(), Direction::West);
        assert_eq!(Direction::East.turn_right(), Direction::South);
        assert_eq!(Direction::West.turn_right(), Direction::North);
        assert_eq!(Direction::NorthEast.turn_right(), Direction::SouthEast);
        assert_eq!(Direction::NorthWest.turn_right(), Direction::NorthEast);
        assert_eq!(Direction::SouthEast.turn_right(), Direction::SouthWest);
        assert_eq!(Direction::SouthWest.turn_right(), Direction::NorthWest);
    }

    #[test]
    fn test_direction_left() {
        assert_eq!(Direction::North.turn_left(), Direction::West);
        assert_eq!(Direction::South.turn_left(), Direction::East);
        assert_eq!(Direction::East.turn_left(), Direction::North);
        assert_eq!(Direction::West.turn_left(), Direction::South);
        assert_eq!(Direction::NorthEast.turn_left(), Direction::NorthWest);
        assert_eq!(Direction::NorthWest.turn_left(), Direction::SouthWest);
        assert_eq!(Direction::SouthEast.turn_left(), Direction::NorthEast);
        assert_eq!(Direction::SouthWest.turn_left(), Direction::SouthEast);
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
    // get tile type as text
    fn get_tile_type_text(&self) -> String;
    // add accessors for contents, is_safe_zone, is_spawn_point, and elevation
    fn is_safe_zone(&self) -> bool;
    fn is_spawn_point(&self) -> bool;
    fn get_type(&self) -> &TileType;
    // Add more common functions as needed
    // is_not_water
    fn is_not_water(&self) -> bool;
}

pub trait EarthTileTrait {
    fn get_elevation(&self) -> Option<u32>;
    // set elevation for Earth tiles
    fn set_elevation(&mut self, elevation: u32);
    // Add more Earth-specific functions as needed
}


#[derive(Clone)]
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

    // get tile type as text
    fn get_tile_type_text(&self) -> String {
        match self.tile_type {
            TileType::Boundary => "Boundary",
            TileType::Mountain => "Mountain",
            TileType::Forest => "Forest",
            TileType::Earth => "Earth",
            TileType::Beach => "Beach",
            TileType::Water => "Water",
            TileType::River => "River",
        }.to_string()
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

    // is_not_water, river, or boundary
    fn is_not_water(&self) -> bool {
        match self.tile_type {
            TileType::Water => false,
            TileType::River => false,
            TileType::Boundary => false,
            _ => true,
        }
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


#[derive(Clone)]
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

    // get tile at x,y
    pub fn get_tile(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x][y]
    }


    pub fn generate_river(&mut self, initial_width: usize) {
        // let (start_x, start_y) = self.find_highest_point();
        // let (end_x, end_y) = self.find_boundary_water_tile();
    }

    pub fn generate_island(&mut self, seed: u32) {
        let perlin = Perlin::new(seed);
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
                debug_println!("Warning: The image buffer is None");
                return;
            }
        };

        let (width, height) = img.dimensions();
        // Check if the circle is within the image dimensions
        if (center.0 + radius > width as i32) || (center.1 + radius > height as i32) {
            debug_println!("Warning: The circle is outside the image dimensions");
            debug_println!("Image dimensions: {} x {}", width, height);
            debug_println!("Circle center: {:?}, radius: {}, color: {:?}", center, radius, color);
            return;
        }

        draw_filled_circle_mut(&mut img, center, radius, color);
    }

    // set a pixel in yellow for every food item in the map
    pub fn draw_food_items(&self, food_items: &MapItemGrid) {
        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                debug_println!("Warning: The image buffer is None");
                return;
            }
        };

        let food_color = Rgb([255, 255, 0]); // yellow
        for x in 0..food_items.items.len() {
            for y in 0..food_items.items[x].len() {
                if let Some(item) = &food_items.items[x][y] {
                    let (x, y) = item.get_position();
                    // if tile type is not water
                    if self.get_tile(x, y).is_not_water() {
                        img.put_pixel(x as u32, y as u32, food_color);
                        // println!("Drawing food item: {} at position: ({}, {})", item.get_item().get_name(), x, y);
                        // display tile type
                        // println!("Tile type: {:?}", self.get_tile(x, y).get_tile_type_text());
                    }
                }
            }
        }

        *self.image_buffer_fg.lock().unwrap() = Some(img);
    }



    pub fn do_draw_circle(&self, x: i32, y: i32) {

        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                debug_println!("Warning: The image buffer is None");
                return;
            }
        };
        let circle_color = COLOR_YELLOW;
        let (width, height) = img.dimensions();
        let center = (x, y);
        let radius = 5;
        // Check if the circle is within the image dimensions
        if (center.0 + radius > width as i32) || (center.1 + radius > height as i32) {
            debug_println!("Warning: The circle is outside the image dimensions");
            debug_println!("Image dimensions: {} x {}", width, height);
            debug_println!("Circle center: {:?}, radius: {}, color: {:?}", center, radius, circle_color);
            return;
        }

        draw_filled_circle_mut(&mut img, center, radius, circle_color);

        let cross_color = COLOR_CYAN;
        let _ = draw_cross(&mut img, cross_color, x, y);

        *self.image_buffer_fg.lock().unwrap() = Some(img);
    }




    fn generate_v_shape_offsets(length: i32, direction: Direction) -> Vec<(i32, i32)> {
        let mut offsets = Vec::new();

        // Generate offsets for the first line
        for i in 0..length {
            offsets.push((-i, i));
        }

        // Generate offsets for the second line
        for i in 1..length { // Start from 1 to avoid duplicating the center point
            offsets.push((i, i));
        }

        // Rotate offsets based on the direction
        for offset in &mut offsets {
            let (dx, dy) = match direction {
                Direction::North => (offset.0, -offset.1),
                Direction::South => (offset.0, offset.1),
                Direction::East => (offset.1, offset.0),
                Direction::West => (-offset.1, offset.0),
                Direction::NorthEast => (offset.1, -offset.0),
                Direction::NorthWest => (-offset.1, -offset.0),
                Direction::SouthEast => (offset.1, offset.0),
                Direction::SouthWest => (-offset.1, offset.0),
            };
            *offset = (dx, dy);
        }

        offsets
    }


    pub fn do_draw_v_shape(&self, x: i32, y: i32, direction: Direction, c: Rgb<u8>) {
        let length = 5;
        let offsets = Grid::generate_v_shape_offsets(length, direction);

        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                debug_println!("Warning: The image buffer is None");
                return;
            }
        };

        let v_shape_color = c;
        let (width, height) = img.dimensions();
        for (dx, dy) in offsets {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                img.put_pixel(x as u32, y as u32, c);
                if x > 0 {
                    img.put_pixel((x - 1) as u32, y as u32, c);
                }
                if x < img.width() as i32 - 1 {
                    img.put_pixel((x + 1) as u32, y as u32, c);
                }
                if y > 0 {
                    img.put_pixel(x as u32, (y - 1) as u32, c);
                }
                if y < img.height() as i32 - 1 {
                    img.put_pixel(x as u32, (y + 1) as u32, c);
                }
            }
        }

        *self.image_buffer_fg.lock().unwrap() = Some(img);
    }

    // save fg image buffer to a png file
    pub fn save_image_fg(&self, filename: &str) {
        // Lock the image buffer and clone it
        let mut img = match self.clone_and_lock_fg() {
            Some(value) => value,
            None => {
                debug_println!("Warning: The image buffer is None");
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
// Characters

#[derive(PartialEq, Debug, Clone)]
pub enum CharacterType {
    Player,
    Human,
    Elf,
    Dwarf,
    Gnole,
    Orc,
    Troll,
    // Add more character types as needed
}

#[derive(Debug, Clone)]
pub struct Character {
    name: String,
    character_type: CharacterType,
    facing: Direction,
    energy: u32,
    health: u32,
    strength: u32,
    agility: u32,
    intelligence: u32,
    x_position: usize,
    y_position: usize,
    my_bag: Bag,
    is_player: bool,
}

impl Character {
    pub fn new(name: String, character_type: CharacterType, health: u32, strength: u32, agility: u32, intelligence: u32, x_position: usize, y_position: usize, is_player: bool) -> Self {
        Character {
            name: proper_name(name),
            character_type,
            health,
            strength,
            agility,
            intelligence,
            x_position,
            y_position,
            energy: 1000,
            my_bag: Bag::new(15),
            facing: Direction::North,
            is_player,
        }
    }


    // Getter method for the character's type
    pub fn get_character_type(&self) -> &CharacterType {
        &self.character_type
    }

    pub fn get_bag_mut(&mut self) -> &mut Bag {
        &mut self.my_bag
    }

    pub fn move_character(&mut self, direction: Direction, grid: &Grid) {

        let (dx, dy) = direction.get_offset();

        // Calculate the next position
        let next_x = (self.x_position as i32 + dx) as usize;
        let next_y = (self.y_position as i32 + dy) as usize;

        // Check if the next position is a boundary
        if let TileType::Boundary = grid.tiles[next_x][next_y].tile_type {
            println!("You cannot move beyond the boundary of this world");
            // turn the character in the opposite direction
            self.facing = direction.opposite();
            // randomly choose to turn left or right
            if rand::random() {
                self.turn_left();
            } else {
                self.turn_right();
            }
            return;
        }

        // Calculate the cost of moving
        let cost = grid.get_cost(self.x_position, self.y_position, next_x, next_y);

        // Check if the character has enough energy to move
        if self.energy < cost as u32 {
            // Check if we have food in our bag
            if let Some(food_item) = self.my_bag.remove_first_food() {
                println!("{} has eaten {} to gain energy", self.name, food_item.get_name());
                self.energy += food_item.get_nutritional_value();
                debug_println!("Energy: {}", self.energy);
            } else {
                println!("{} does not have enough energy to move.", self.name);
                println!("You will rest this turn");
                self.energy += 20;
                return;
            }
        }

        // Deduct the cost from the character's energy
        self.energy -= cost as u32;

        // Display the energy level and cost of moving
        debug_println!("Energy: {}, Cost: {}", self.energy, cost);

        // Update the character's position
        self.x_position = next_x;
        self.y_position = next_y;
    }

    // move forward in the direction we are facing
    pub fn move_forward(&mut self, grid: &Grid) {
        self.move_character(self.facing, grid);
    }

    // move backward in the opposite direction we are facing
    pub fn move_backward(&mut self, grid: &Grid) {
        self.move_character(self.facing.opposite(), grid);
    }


    // turn the character left or right
    pub fn turn_character(&mut self, direction: Direction) {
        self.facing = direction;
    }

    // turn left
    pub fn turn_left(&mut self) {
        self.facing = self.facing.turn_left();
    }

    // turn right
    pub fn turn_right(&mut self) {
        self.facing = self.facing.turn_right();
    }


    pub fn teleport_character(&mut self, x: usize, y: usize) {
        self.x_position = x;
        self.y_position = y;
    }

    pub fn automate(&mut self, grid: &Grid, items: &mut MapItemGrid) {
        if self.is_player {
            return;
        }


        let valid_neighbors = self.get_valid_neighbors(grid);

        if let Some(food_location) = self.find_food(&valid_neighbors, items) {
            self.move_to_food(food_location, items);
            self.move_forward_if_possible(grid);
        } else {
            self.move_forward_if_possible(grid);
        }
    }

    fn get_valid_neighbors(&self, grid: &Grid) -> Vec<(usize, usize)> {
        grid.get_neighbors(self.x_position, self.y_position)
            .into_iter()
            .filter(|(x, y)| grid.get_tile(*x, *y).is_not_water())
            .collect()
    }

    fn find_food(&self, valid_neighbors: &[(usize, usize)], items: &MapItemGrid) -> Option<(usize, usize)> {
        valid_neighbors.iter().find(|(x, y)| {
            items.items[*x][*y].as_ref().map_or(false, |item| item.get_item().is_food())
        }).copied()
    }

    fn move_to_food(&mut self, food_location: (usize, usize), items: &mut MapItemGrid) {
        // display current position
        debug_println!("{} is at position: ({}, {})", self.name, self.x_position, self.y_position);
        // display moving towards food location
        debug_println!("{} is moving towards food at position: ({}, {})", self.name, food_location.0, food_location.1);
        let dx = food_location.0 as i32 - self.x_position as i32;
        let dy = food_location.1 as i32 - self.y_position as i32;
        // display direction we are facing
        debug_println!("{} is facing: {:?}", self.name, self.facing);
        self.facing = Direction::from_offset(dx, dy).unwrap_or(self.facing);
        // display direction we are facing after turning
        debug_println!("{} is facing: {:?}", self.name, self.facing);
        // what is the distance between the character and the food
        let distance = ((dx * dx + dy * dy) as f64).sqrt();
        if(distance < 2.0) {
            self.teleport_character(food_location.0, food_location.1);
            self.pick_up_food(food_location, items);
            return;
        }
        if dx == 0 && dy == 0 {
            self.pick_up_food(food_location, items);
        }

    }

    fn pick_up_food(&mut self, food_location: (usize, usize), items: &mut MapItemGrid) {
        if let Some(map_item) = items.items[food_location.0][food_location.1].take() {
            if let Some(food_item) = map_item.item.as_any().downcast_ref::<FoodItem>() {
                self.my_bag.add_item(Box::new(food_item.clone()));
                println!("{} has picked up a food item", self.name);
                if self.my_bag.is_full() {
                    println!("{}'s bag is full", self.name);
                    // find least nutritious food item and eat it
                    if let Some(food_item) = self.my_bag.remove_least_nutritious_food() {
                        println!("{} has eaten {} to make space in the bag", self.name, food_item.get_name());
                    }
                }

                self.facing = if rand::random() { self.facing.turn_left() } else { self.facing.turn_right() };
            }
        }
    }

    fn move_forward_if_possible(&mut self, grid: &Grid) {
        self.move_forward(grid);
        if let TileType::Boundary | TileType::Water = grid.get_tile(self.x_position, self.y_position).tile_type {
            self.move_away_from_obstacle();
            self.move_forward(grid);
        }
    }

    fn move_away_from_obstacle(&mut self) {
        self.facing = self.facing.opposite();
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

    // get character by index
    pub fn get_character_by_index(&self, index: usize) -> Option<&Character> {
        self.characters.get(index)
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

    // automate all the computer controlled characters in the character manager
    pub fn automate_all(&mut self, grid: &Grid, items: &mut MapItemGrid) {
        for character in &mut self.characters {
            // automate character depending on type
            character.automate(grid, items);
        }
    }
}

#[cfg(test)]
mod character_tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, true);
        assert_eq!(*character.get_character_type(), CharacterType::Human);
    }

    #[test]
    fn test_teleport_character() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, true);
        character.teleport_character(5, 5);
        assert_eq!(character.x_position, 5);
        assert_eq!(character.y_position, 5);
    }


    #[test]
    fn test_character_manager_creation() {
        let character_manager = CharacterManager::new();
        assert_eq!(character_manager.count_characters(), 0);
    }

    #[test]
    fn test_add_character() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        character_manager.add_character(character);
        assert_eq!(character_manager.count_characters(), 1);
    }

    #[test]
    fn test_remove_character() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        character_manager.add_character(character);
        let removed_character = character_manager.remove_character(0);
        assert_eq!(removed_character.is_some(), true);
        assert_eq!(character_manager.count_characters(), 0);
    }

    #[test]
    fn test_get_character() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        character_manager.add_character(character);
        let retrieved_character = character_manager.get_character_by_index(0);
        assert_eq!(retrieved_character.is_some(), true);
    }

    #[test]
    fn test_get_character_by_name() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        character_manager.add_character(character);
        let retrieved_character = character_manager.get_character_by_name("Player");
        assert_eq!(retrieved_character.is_some(), true);
    }

    #[test]
    fn test_get_characters_at_position() {
        let mut character_manager = CharacterManager::new();
        let character1 = Character::new("Player1".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        let character2 = Character::new("Player2".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        character_manager.add_character(character1);
        character_manager.add_character(character2);
        let characters_at_position = character_manager.get_characters_at_position(0, 0);
        assert_eq!(characters_at_position.len(), 2);
    }

    #[test]
    fn test_is_character_at_position() {
        let mut character_manager = CharacterManager::new();
        let character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        character_manager.add_character(character);
        let is_character_at_position = character_manager.is_character_at_position(0, 0);
        assert_eq!(is_character_at_position, true);
    }

    // test move north
    #[test]
    fn test_move_north() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 5, 5, true);
        let grid = Grid::new(20);
        character.move_character(Direction::North, &grid);
        assert_eq!(character.x_position, 5);
        assert_eq!(character.y_position, 4);
    }

    // test move south
    #[test]
    fn test_move_south() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 5, 5, false);
        let grid = Grid::new(20);
        character.move_character(Direction::South, &grid);
        assert_eq!(character.x_position, 5);
        assert_eq!(character.y_position, 6);
    }

    // test move east
    #[test]
    fn test_move_east() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 5, 5, false);
        let grid = Grid::new(20);
        character.move_character(Direction::East, &grid);
        assert_eq!(character.x_position, 6);
        assert_eq!(character.y_position, 5);
    }

    // test move west
    #[test]
    fn test_move_west() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 5, 5, false);
        let grid = Grid::new(20);
        character.move_character(Direction::West, &grid);
        assert_eq!(character.x_position, 4);
        assert_eq!(character.y_position, 5);
    }


    // test move to boundary
    #[test]
    fn test_move_to_boundary() {
        let mut character = Character::new("Player".to_string(), CharacterType::Human, 100, 10, 10, 10, 0, 0, false);
        let grid = Grid::new(10);
        character.move_character(Direction::East, &grid);
        assert_eq!(character.x_position, 1);
        assert_eq!(character.y_position, 0);
    }
}


// =================================================================================================
// interactive command line interpreter

// define commands
#[derive(Debug, PartialEq)]
pub enum Command {
    Move(Direction),
    Run(Direction),
    Rest,
    EatItemByName(String),
    GetItemByName(String),
    DisplayItemsAtXY(usize, usize),
    TurnLeft,
    TurnRight,
    MoveForward,
    MoveBackward,
    Teleport(usize, usize),
    ListCharacters,
    ListItems,
    LookAround,
    ShowItem,
    ShowCharacter,
    ShowNamedCharacter(String),
    ShowMap,
    Quit,
    Help,
    Unknown,
}


// parse subcommand for items
fn parse_item_command(input: &str) -> Command {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    match parts[0].to_lowercase().as_str() {

        _ => Command::Unknown,
    }
}


// parse command
fn parse_command(input: &str) -> Command {

    // define a vector of noise words
    let noise_words = vec!["in", "at", "to", "the", "a", "an", ","];

    // check if the input is empty
    if input.trim().is_empty() {
        return Command::Unknown;
    }

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    // filter out noise words
    let parts: Vec<&str> = parts.iter().filter(|&part| !noise_words.contains(&part)).map(|&part| part).collect();

    match parts[0].to_lowercase().as_str() {
        "move" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                if let Some(direction) =
                    Direction::from_lower_case_str(parts[1].to_lowercase().as_str()) {
                    Command::Move(direction)
                } else {
                    Command::Unknown
                }
            }
        }
        "run" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                if let Some(direction) =
                    Direction::from_lower_case_str(parts[1].to_lowercase().as_str()) {
                    Command::Run(direction)
                } else {
                    Command::Unknown
                }
            }
        }
        "turn" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                match parts[1].to_lowercase().as_str() {
                    "left" => Command::TurnLeft,
                    "right" => Command::TurnRight,
                    _ => Command::Unknown,
                }
            }
        }
        "rest" => Command::Rest,
        "fd" => Command::MoveForward, // alias for "forward"
        "forward" => Command::MoveForward,
        "backward" => Command::MoveBackward,

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
                parse_item_command(parts[1])
            }
        }

        "eat" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                Command::EatItemByName(parts[1].to_string())
            }
        }

        "get" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                Command::GetItemByName(parts[1].to_string())
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
                    "bag" => Command::ListItems, // alias for "list items
                    "item" => Command::ShowItem,
                    "me" | "character" => Command::ShowCharacter,
                    "map" => Command::ShowMap,

                    // assume the player wants to see the character named <name>
                    _ => Command::ShowNamedCharacter(parts[1].to_string())
                }
            }
        }
        "me" => Command::ShowCharacter, // alias for "show character"

        // look around
        "look" => {
            if parts.len() < 2 {
                Command::Unknown
            } else {
                match parts[1].to_lowercase().as_str() {
                    "around" => Command::LookAround,
                    _ => Command::Unknown,
                }
            }
        }
        "see" => Command::LookAround, // alias for "look around"
        "quit" => Command::Quit,
        "help" => Command::Help,

        // display items from MapItems at x,y
        "display" => {
            if parts.len() < 3 {
                Command::Unknown
            } else {
                let x = parts[1].parse().unwrap_or(0);
                let y = parts[2].parse().unwrap_or(0);
                Command::DisplayItemsAtXY(x, y)
            }
        }
        _ => Command::Unknown,
    }
}


// execute a command
fn execute_command(command: Command, manager: &mut CharacterManager, grid: &mut Grid, items: &mut MapItemGrid, character_index: usize) {
    match command {
        Command::Move(direction) => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.move_character(direction, grid);
        }
        Command::Run(direction) => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            dotimes!(4, {
                character.move_character(direction, grid);
            });
        }
        Command::Teleport(x, y) => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.teleport_character(x, y);
        }

        Command::ListCharacters => {
            manager.list_characters();
        }
        Command::ListItems => {
            command_show_bag(manager, character_index);
        }
        Command::ShowItem => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            if let Some(item) = character.my_bag.items.first() {
                item.show_item();
            }
        }
        Command::ShowCharacter => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            println!("{:?}", character);
        }
        Command::ShowMap => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            command_show_map(grid, items, character);
        }
        Command::Quit => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        Command::Help => {
            command_help();
        }
        Command::Unknown => {
            println!("Unknown command. Type 'help' to see available commands.");
        }
        Command::TurnLeft => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.turn_left();
        }
        Command::TurnRight => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.turn_right();
        }
        Command::MoveForward => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.move_forward(grid);
        }
        Command::MoveBackward => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.move_backward(grid);
        }
        Command::DisplayItemsAtXY(x, y) => {
            command_display_items_at_xy(items, x, y);
        }
        // looks around the neighboring tiles for items
        Command::LookAround => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            command_look_around(items, &mut character);
        }
        Command::EatItemByName(item_name) => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            command_eat_item_by_name(&mut character, item_name);
        }

        Command::GetItemByName(item_name) => {
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            command_get_item_by_name(items, character, item_name);
        }
        Command::ShowNamedCharacter(item_name) => {
            command_show_some_thing_by_name(manager, items, &item_name);
        }
        Command::Rest => {
            // resting restores energy at the expense of time
            let mut character: &mut Character = manager.get_character_mut(character_index).unwrap();
            character.energy += 20;
        }
    }
}

fn command_display_items_at_xy(items: &mut MapItemGrid, x: usize, y: usize) {
    let tile_items = items.get_items_at(x, y);
    if tile_items.is_empty() {
        println!("No items found at position: ({}, {})", x, y);
    } else {
        println!("Items found at position: ({}, {})", x, y);
        for item in tile_items {
            println!("Item: {}", item.get_item().get_name());
        }
    }
}


fn command_show_bag(manager: &mut CharacterManager, character_index: usize) {
// check if bag is empty
    let mut player: &mut Character = manager.get_character_mut(character_index).unwrap();
    if player.my_bag.items.is_empty() {
        println!("No items found in the bag");
    } else {
        // display items in the player's bag
        player.my_bag.items.iter().for_each(|item| item.show_item());
    }
}

fn command_show_some_thing_by_name(manager: &mut CharacterManager, items: &mut MapItemGrid, item_name: &String) {
    // Try to find a character with the given name
    let item_name = proper_name(item_name.clone());
    if let Some(character) = manager.get_character_by_name(item_name.as_str()) {
        println!("{:?}", character);
        return;
    }

    // If no character was found, try to find an item on the current tile
    let player: &mut Character = manager.get_character_mut(0).unwrap();
    let tile_items = items.get_items_at(player.x_position, player.y_position);
    let found_item = tile_items.iter().find(|item| {
        item.get_item().get_name().to_lowercase() == item_name.to_lowercase()
    });

    if let Some(item) = found_item {
        item.get_item().show_item();
        return;
    }

    // If no item was found on the tile, try to find an item in the player's bag
    let bag_item = player.my_bag.items.iter().find(|item| {
        item.get_name().to_lowercase() == item_name.to_lowercase()
    });

    if let Some(item) = bag_item {
        item.show_item();
        return;
    }

    // If no character or item was found, print a message
    println!("No character or item found with the name: {}", item_name);
}

fn command_help() {
    println!("Several commands on one line; may be seperated by semicolons.");

    println!("Movement:");
    println!("move <direction> - Move the player character in the specified direction (north, south, east, west)");
    println!("turn <direction> - Turn the player character in the specified direction (left, right)");
    println!("forward (fd) - Move the player character forward in the direction they are facing");
    println!("backward (bd) - Move the player character backward in the opposite direction they are facing");

    // look around
    println!("look around - Look around to see items, and others nearby");


    // topic items
    println!("Items");
    println!("get <item> - Add an item found *here* to your bag (apple, banana, orange, shovel etc..)");

    println!("list items - List all items in the bag");
    println!("show item - Show the first item in the player character's bag");


    println!("show character - Show the player character's details");
    println!("show map - Show the map with the player character's position");

    println!("God like powers:");
    println!("display <x> <y> - Display items at the specified position");
    println!("list characters - List all characters in the character manager");
    println!("teleport <x> <y> - Teleport the player character to the specified position");


    println!("System commands:");
    println!("quit - Quit the program");
    println!("help - Show available commands");
}

fn command_eat_item_by_name(player: &mut &mut Character, item_name: String) {
    let item = player.my_bag.items.iter().find(|item| {
        let item_name_lowercase = item.get_name().to_lowercase();
        let input_name_lowercase = item_name.to_lowercase();
        item_name_lowercase == input_name_lowercase
    });

    if let Some(item) = item {
        println!("Eating item: {}", item.get_name());
        // check if item is food
        if item.is_food() {
            // increase player's energy
            player.energy += item.get_nutritional_value();
            println!("Energy increased by: {}", item.get_nutritional_value());
        } else {
            println!("Item is not food");
            return;
        }


        // remove item from the bag (lowercase the comparison)
        player.my_bag.items.retain(|item| *item.get_name().to_lowercase() != item_name.to_lowercase());
    } else {
        println!("Item not found in the bag");
    }
}

fn command_look_around(items: &mut MapItemGrid, player: &mut &mut Character) {
// say "taking a look around"
    println!("Taking a look around ...");
    // display items around the player character
    let x = player.x_position;
    let y = player.y_position;
    let mut found_items = false;
    for dx in -1..=1 {
        for dy in -1..=1 {
            let tile_items = items.get_items_at((x as i32 + dx) as usize, (y as i32 + dy) as usize);
            if !tile_items.is_empty() {
                found_items = true;
                println!("Items found nearby at position: ({}, {})", x as i32 + dx, y as i32 + dy);

                // compare x,y with dx, dy, and work out the direction
                let direction = match (dx, dy) {
                    (0, -1) => "north",
                    (1, -1) => "northeast",
                    (1, 0) => "east",
                    (1, 1) => "southeast",
                    (0, 1) => "south",
                    (-1, 1) => "southwest",
                    (-1, 0) => "west",
                    (-1, -1) => "northwest",
                    (0, 0) => "here", // player position
                    _ => "unknown",
                };
                println!("Direction of item: {}", direction);

                for item in tile_items {
                    println!("Item: {}", item.get_item().get_name());
                }
            }
        }
    }
}

fn command_get_item_by_name(items: &mut MapItemGrid, mut player: &mut Character, item_name: String) {
// find the items at the players location
    let tile_items = items.get_items_at(player.x_position, player.y_position);
    let item = tile_items.iter().find(|item| {
        let item_name_lowercase = item.get_item().get_name().to_lowercase();
        let input_name_lowercase = item_name.to_lowercase();
        item_name_lowercase == input_name_lowercase
    });
    // if item is found add to player's bag
    if let Some(item) = item {
        println!("Getting item: {}", item.get_item().get_name());
        player.my_bag.items.push(item.get_item().clone());
        // remove named item from map
        let item_name = item.get_item().get_name().clone();
        items.remove_named_item_at(player.x_position, player.y_position, &item_name);
    } else {
        println!("Item not found at position: ({}, {})", player.x_position, player.y_position);
    }
}


fn command_show_map(grid: &mut Grid, items: &mut MapItemGrid, mut player: &mut Character) {
    grid.clear_image_fg();
    grid.draw_food_items(&items);
    grid.do_draw_circle(player.x_position as i32, player.y_position as i32);
    grid.save_image_fg("map.png");
    // tell user we have generated the map.
    println!("Map generated. Check the map.png file for the map.");
}

// command loop
fn command_loop(manager: &mut CharacterManager, grid: &mut Grid, items: &mut MapItemGrid) {
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
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(input) => {
                let commands: Vec<&str> = input.trim().split(";").collect();
                for command in commands {
                    if command.trim().is_empty() {
                        manager.automate_all(grid, items);
                        continue;
                    }
                    let command = parse_command(command);
                    execute_command(command, manager, grid, items, 0);
                }
                manager.automate_all(grid, items);
                println!("Enter command: ");
            }
            Err(_) => { manager.automate_all(grid, items); }
        }
    }
}

// test commands
#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let command = parse_command("move north");
        assert_eq!(command, Command::Move(Direction::North));
    }

    #[test]
    fn test_parse_command_empty() {
        let command = parse_command("");
        assert_eq!(command, Command::Unknown);
    }

    #[test]
    fn test_parse_command_unknown() {
        let command = parse_command("unknown");
        assert_eq!(command, Command::Unknown);
    }


    #[test]
    fn test_parse_command_noise_words() {
        let command = parse_command("move to the north");
        assert_eq!(command, Command::Move(Direction::North));
    }

    #[test]
    fn test_parse_command_look_around() {
        let command = parse_command("look around");
        assert_eq!(command, Command::LookAround);
    }


    #[test]
    fn test_parse_command_get_item() {
        let command = parse_command("get apple");
        assert_eq!(command, Command::GetItemByName("apple".to_string()));
    }

    #[test]
    fn test_parse_command_get_item_noise_words() {
        let command = parse_command("get an apple");
        assert_eq!(command, Command::GetItemByName("apple".to_string()));
    }
}


// main function
// Things get a little hairy here, but we are just initializing the grid, items, and character manager
// and starting the command loop

fn main() {
    println!("Processing map ... ");

    // mutable grid
    let mut grid = GRID.lock().unwrap();
    let mut items = MapItemGrid::new(2048);
    let mut manager = CharacterManager::new();

    let player = Character::new("Kevin".to_string(),
                                CharacterType::Player,
                                100,
                                10,
                                10,
                                10,
                                500,
                                500, true);

    manager.add_character(player);

    // add troll
    let troll = Character::new("Troll".to_string(),
                               CharacterType::Troll,
                               100,
                               10,
                               10,
                               10,
                               1000,
                               1000, false);

    manager.add_character(troll);
    items.add_random_food_items(28000, &mut grid);
    items.add_random_useful_items(1000, &mut grid);


    grid.generate_island(17);

    grid.set_boundary_margin(5);
    grid.generate_elevation_png("elevation.png");


    // start the command loop
    // display "ready"
    println!("Ready!");

    command_loop(&mut manager, &mut grid, &mut items);
}

// =================================================================================================
// <EOT>