pub mod bsp_dungeon;
pub mod common;
pub mod simple_map;
pub mod bsp_interior_builder;
pub mod cellular_automata_builder;

use rltk::Point;
use specs::{World};

use super::Map;

pub trait MapBuilder {
    fn build_map(&mut self, ecs: &mut World);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Point;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(cellular_automata_builder::CellularAutomataBuilder::new(new_depth))
}
