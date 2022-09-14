use std::collections::HashSet;

use rltk::{RGB, Point, RandomNumberGenerator, Rect};
use specs::prelude::*;

use crate::{components::{Position, Renderable, Viewshed, Name, CombatStats, Monster, BlocksTile, Item, ProvidesHealing, Consumable, Ranged, InflictsDamage}, player::Player, map::MAPWIDTH};

pub const MAX_MONSTERS: i32 = 4;
pub const MAX_ITEMS: i32 = 2;

pub fn spawn_player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {point: Point::from_tuple((x, y))})
        .with(Renderable {
            glyph: rltk::to_cp437('☻'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: HashSet::new(),
            range: 8,
        })
        .with(Name {name: "Player".to_string()})
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { orc(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }
}

fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), "Goblin"); }

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, name : S) {
    ecs.create_entity()
        .with(Position{ point: Point::new(x, y) })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed{ visible_tiles : HashSet::new(), range: 8})
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points = HashSet::new();
    let mut item_spawn_points = HashSet::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _ in 0..num_monsters {
            let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1 - 1))) as usize;
            let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1 - 1))) as usize;
            let idx = (y * MAPWIDTH) + x;
            monster_spawn_points.insert(idx);
        }
        for _ in 0..num_items {
            let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1 - 1))) as usize;
            let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1 - 1))) as usize;
            let idx = (y * MAPWIDTH) + x;
            item_spawn_points.insert(idx);
        }
    }

    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }
    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_item(ecs, x as i32, y as i32);
    }
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {point: Point::new(x, y)})
        .with(Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {name: "Health Potion".to_string()})
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing {heal_amount: 8})
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {point: Point::new(x, y)})
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {name: "Magic Missile Scroll".to_string()})
        .with(Item{})
        .with(Consumable{})
        .with(Ranged {range: 6})
        .with(InflictsDamage {damage: 8})
        .build();
}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => {
            return health_potion(ecs, x, y);
        }
        _ => {
            return magic_missile_scroll(ecs, x, y);
        }
    }
}