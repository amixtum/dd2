use rltk::Point;
use specs::prelude::*;

use crate::{gamelog::GameLog, components::{WantsToDropItem, Name, Position, InBackpack}};

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut log,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropper_pos: Point;
            {
                let dropped_at = positions.get(entity).unwrap();
                dropper_pos = dropped_at.point;
            }
            positions.insert(to_drop.item, Position { point: dropper_pos }).expect(&format!("Unable to insert position with (x, y) = ({}, {})", dropper_pos.x, dropper_pos.y));
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                log.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}