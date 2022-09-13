use rltk::console;
use specs::prelude::*;

use crate::{components::{CombatStats, SufferDamage, Position, Name}, player::Player, map::Map, gamelog::GameLog};

pub struct DamageSystem { }

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_dead(ecs: &mut World) {
    let mut dead = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                if let Some(_) = players.get(entity) {
                    console::log("You are dead")
                } else {
                    if let Some(victim_name) = names.get(entity) {
                        log.entries.push(format!("{} is dead", &victim_name.name));
                    }
                    dead.push(entity);
                }
            }
        }
    }

    for victim in dead.iter_mut() {
        unblock_dead(ecs, victim);

        ecs.delete_entity(*victim).expect("Unable to delete");
    }
}

pub fn unblock_dead(ecs: &mut World, entity: &Entity) {
    let mut map = ecs.write_resource::<Map>();
    let positions = ecs.read_storage::<Position>();
    if let Some(pos) = positions.get(*entity) {
        map.blocked_tiles.remove(&pos.point);
    }   
}