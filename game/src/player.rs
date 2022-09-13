use std::cmp::{min, max};

use rltk::{VirtualKeyCode, Rltk, Point};

use specs::prelude::*;
use specs_derive::Component;

use crate::components::{CombatStats, WantsToMelee, Viewshed};
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
            VirtualKeyCode::Semicolon => {
                return RunState::Looking;
            }
            _ => { 
                return RunState::AwaitingInput;
            },
        }
    }

    RunState::PlayerTurn
}

fn try_move_cursor(delta_x: i32, delta_y: i32, gs: &mut State) -> (i32, i32) {
    let map = gs.ecs.fetch::<Map>();
    let new_pos = Point::new(gs.look_cursor.0 + delta_x, gs.look_cursor.1 + delta_y);

    if new_pos.x <= 0 || new_pos.x >= map.width || new_pos.y <= 0 || new_pos.y >= map.height {
        return gs.look_cursor;
    }

    let player = gs.ecs.fetch::<Entity>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    if let Some(viewshed) = viewsheds.get(*player) {
        if viewshed.visible_tiles.contains(&new_pos) {
            return (new_pos.x, new_pos.y)
        }
    }
    gs.look_cursor
}

pub fn look_mode_input(gs: &mut State, ctx: &mut Rltk) -> (RunState, (i32, i32)) {
    match ctx.key {
        None => {
            return (RunState::Looking, gs.look_cursor);
        },
        Some(key) => match key {
            VirtualKeyCode::A |
            VirtualKeyCode::H => {
                return (RunState::Looking, try_move_cursor(-1, 0, gs));
            },
            VirtualKeyCode::D |
            VirtualKeyCode::L => {
                return (RunState::Looking, try_move_cursor(1, 0, gs));
            },
            VirtualKeyCode::W |
            VirtualKeyCode::K => {
                return (RunState::Looking, try_move_cursor(0, -1, gs));
            },
            VirtualKeyCode::S |
            VirtualKeyCode::J => {
                return (RunState::Looking, try_move_cursor(0, 1, gs));
            },
            VirtualKeyCode::Q |
            VirtualKeyCode::Y => {
                return (RunState::Looking, try_move_cursor(-1, -1, gs));
            },
            VirtualKeyCode::E |
            VirtualKeyCode::U => {
                return (RunState::Looking, try_move_cursor(1, -1, gs));
            },
            VirtualKeyCode::Z |
            VirtualKeyCode::B => {
                return (RunState::Looking, try_move_cursor(-1, 1, gs));
            },
            VirtualKeyCode::C |
            VirtualKeyCode::N => {
                return (RunState::Looking, try_move_cursor(1, 1, gs));
            },
            VirtualKeyCode::Semicolon => {
                return (RunState::CleanupTooltips, (-1, -1));
            }
            _ => { 
                return (RunState::Looking, gs.look_cursor);
            },
        }
    }
}