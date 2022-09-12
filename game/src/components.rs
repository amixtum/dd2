use std::collections::HashSet;

use rltk::{Point, RGB};

use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Position {
    pub point: Point,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: HashSet<rltk::Point>,
    pub range: i32,
}

#[derive(Component, Debug)]
pub struct Monster {}