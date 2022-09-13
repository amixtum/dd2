use std::cmp::{min, max};

use rltk::{VirtualKeyCode, Rltk, Point};

use specs::prelude::*;
use specs_derive::Component;

use crate::components::{CombatStats, WantsToMelee};
use crate::map::{Map};
use crate::state::RunState;

use super::components::Position;
use super::state::State;

#[derive(Component, Debug)]
pub struct Player { }

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, position) in (&entities, &mut players, &mut positions).join() {
        let new_x = position.point.x + delta_x;
        let new_y = position.point.y + delta_y;
        if new_x < 1 || new_x > map.width - 1 || new_y < 1 || new_y > map.height - 1 { 
            return; 
        }

        let dest_idx = map.xy_flat(new_x, new_y);

        for potential_target in map.tile_content[dest_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {},
                Some(_target) => {
                    wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).expect("Add target failed");
                    return;
                }
            }
        }

        if !map.blocked_tiles.contains(&Point::new(new_x, new_y)) {
            position.point.x = min(79, max(0, new_x));
            position.point.y = min(49, max(0, new_y));

            let mut player_pos = ecs.write_resource::<Point>();
            player_pos.x = position.point.x;
            player_pos.y = position.point.y;
        }
    }


}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => {
            return RunState::AwaitingInput;
        },
        Some(key) => match key {
            VirtualKeyCode::A |
            VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs);
            },
            VirtualKeyCode::D |
            VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs);
            },
            VirtualKeyCode::W |
            VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs);
            },
            VirtualKeyCode::S |
            VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs);
            },
            VirtualKeyCode::Q |
            VirtualKeyCode::Y => {
                try_move_player(-1, -1, &mut gs.ecs);
            },
            VirtualKeyCode::E |
            VirtualKeyCode::U => {
                try_move_player(1, -1, &mut gs.ecs);
            },
            VirtualKeyCode::Z |
            VirtualKeyCode::B => {
                try_move_player(-1, 1, &mut gs.ecs);
            },
            VirtualKeyCode::C |
            VirtualKeyCode::N => {
                try_move_player(1, 1, &mut gs.ecs);
            },
            _ => { 
                return RunState::AwaitingInput;
            },
        }
    }

    RunState::Running
}