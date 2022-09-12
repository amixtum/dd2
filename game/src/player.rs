use std::cmp::{min, max};

use rltk::{VirtualKeyCode, Rltk};

use specs::prelude::*;
use specs_derive::Component;

use crate::map::{TileType, Map};

use super::components::Position;
use super::state::State;

#[derive(Component, Debug)]
pub struct Player { }

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, position) in (&mut players, &mut positions).join() {
        let dest_idx = map.xy_flat(position.point.x + delta_x, position.point.y + delta_y);
        if map.tiles[dest_idx] != TileType::Wall {
            position.point.x = min(79, max(0, position.point.x + delta_x));
            position.point.y = min(49, max(0, position.point.y + delta_y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {},
        Some(key) => match key {
            VirtualKeyCode::A => {
                try_move_player(-1, 0, &mut gs.ecs);
                gs.needs_redraw = true;
            },
            VirtualKeyCode::D => {
                try_move_player(1, 0, &mut gs.ecs);
                gs.needs_redraw = true;
            },
            VirtualKeyCode::W => 
            {
                try_move_player(0, -1, &mut gs.ecs);
                gs.needs_redraw = true;
            },
            VirtualKeyCode::S => {
                try_move_player(0, 1, &mut gs.ecs);
                gs.needs_redraw = true;
            },
            _ => { },
        }
    }
}