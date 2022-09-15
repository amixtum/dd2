use std::cmp::{max, min};

use rltk::{Point, Rltk, VirtualKeyCode};

use specs::prelude::*;
use specs_derive::Component;

use crate::components::{CombatStats, Item, Viewshed, WantsToMelee, WantsToPickUpItem};
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::state::RunState;

use super::components::Position;
use super::state::State;

#[derive(Component, Debug)]
pub struct Player {}

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
                None => {}
                Some(_target) => {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
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

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut game_log = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.point == *player_pos {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => {
            game_log
                .entries
                .push("There is nothing here to pickup".to_string());
        }
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickUpItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickUpItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pick up");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => {
            return RunState::AwaitingInput;
        }
        Some(key) => match key {
            VirtualKeyCode::A | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs);
            }
            VirtualKeyCode::D | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs);
            }
            VirtualKeyCode::W | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs);
            }
            VirtualKeyCode::S | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs);
            }
            VirtualKeyCode::Q | VirtualKeyCode::Y => {
                try_move_player(-1, -1, &mut gs.ecs);
            }
            VirtualKeyCode::E | VirtualKeyCode::U => {
                try_move_player(1, -1, &mut gs.ecs);
            }
            VirtualKeyCode::Z | VirtualKeyCode::B => {
                try_move_player(-1, 1, &mut gs.ecs);
            }
            VirtualKeyCode::C | VirtualKeyCode::N => {
                try_move_player(1, 1, &mut gs.ecs);
            }
            VirtualKeyCode::Semicolon => {
                return RunState::Looking;
            }
            VirtualKeyCode::G => {
                get_item(&mut gs.ecs);
            }
            VirtualKeyCode::I => {
                return RunState::ShowInventory;
            }
            VirtualKeyCode::Minus => {
                return RunState::ShowDropItem;
            }
            _ => {
                return RunState::AwaitingInput;
            }
        },
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
            return (new_pos.x, new_pos.y);
        }
    }
    gs.look_cursor
}

pub fn look_mode_input(gs: &mut State, ctx: &mut Rltk) -> (RunState, (i32, i32)) {
    match ctx.key {
        None => {
            return (RunState::Looking, gs.look_cursor);
        }
        Some(key) => match key {
            VirtualKeyCode::A | VirtualKeyCode::H => {
                return (RunState::Looking, try_move_cursor(-1, 0, gs));
            }
            VirtualKeyCode::D | VirtualKeyCode::L => {
                return (RunState::Looking, try_move_cursor(1, 0, gs));
            }
            VirtualKeyCode::W | VirtualKeyCode::K => {
                return (RunState::Looking, try_move_cursor(0, -1, gs));
            }
            VirtualKeyCode::S | VirtualKeyCode::J => {
                return (RunState::Looking, try_move_cursor(0, 1, gs));
            }
            VirtualKeyCode::Q | VirtualKeyCode::Y => {
                return (RunState::Looking, try_move_cursor(-1, -1, gs));
            }
            VirtualKeyCode::E | VirtualKeyCode::U => {
                return (RunState::Looking, try_move_cursor(1, -1, gs));
            }
            VirtualKeyCode::Z | VirtualKeyCode::B => {
                return (RunState::Looking, try_move_cursor(-1, 1, gs));
            }
            VirtualKeyCode::C | VirtualKeyCode::N => {
                return (RunState::Looking, try_move_cursor(1, 1, gs));
            }
            VirtualKeyCode::Semicolon => {
                return (RunState::CleanupTooltips, (-1, -1));
            }
            _ => {
                return (RunState::Looking, gs.look_cursor);
            }
        },
    }
}

fn try_move_ranged_cursor(
    delta_x: i32,
    delta_y: i32,
    cursor: Point,
    range: i32,
    gs: &mut State,
) -> Point {
    let map = gs.ecs.fetch::<Map>();
    let player_pos = gs.ecs.fetch::<Point>();
    let new_pos = Point::new(cursor.x + delta_x, cursor.y + delta_y);

    if new_pos.x <= 0 || new_pos.x >= map.width || new_pos.y <= 0 || new_pos.y >= map.height {
        return cursor;
    }

    let player = gs.ecs.fetch::<Entity>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    if let Some(viewshed) = viewsheds.get(*player) {
        if viewshed.visible_tiles.contains(&new_pos) {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, new_pos);
            if dist <= range as f32 {
                return new_pos;
            }
        }
    }

    cursor
}

pub fn ranged_targeting_input(gs: &mut State, ctx: &mut Rltk, cursor: Point, range: i32) -> Point {
    match ctx.key {
        None => {
            return cursor;
        }
        Some(key) => match key {
            VirtualKeyCode::A | VirtualKeyCode::H => {
                return try_move_ranged_cursor(-1, 0, cursor, range, gs);
            }
            VirtualKeyCode::D | VirtualKeyCode::L => {
                return try_move_ranged_cursor(1, 0, cursor, range, gs);
            }
            VirtualKeyCode::W | VirtualKeyCode::K => {
                return try_move_ranged_cursor(0, -1, cursor, range, gs);
            }
            VirtualKeyCode::S | VirtualKeyCode::J => {
                return try_move_ranged_cursor(0, 1, cursor, range, gs);
            }
            VirtualKeyCode::Q | VirtualKeyCode::Y => {
                return try_move_ranged_cursor(-1, -1, cursor, range, gs);
            }
            VirtualKeyCode::E | VirtualKeyCode::U => {
                return try_move_ranged_cursor(1, -1, cursor, range, gs);
            }
            VirtualKeyCode::Z | VirtualKeyCode::B => {
                return try_move_ranged_cursor(-1, 1, cursor, range, gs);
            }
            VirtualKeyCode::C | VirtualKeyCode::N => {
                return try_move_ranged_cursor(1, 1, cursor, range, gs);
            }
            _ => {
                return cursor;
            }
        },
    }
}
