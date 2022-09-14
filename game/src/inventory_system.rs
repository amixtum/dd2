use specs::prelude::*;

use crate::{gamelog::GameLog, components::{WantsToPickUpItem, Position, Name, InBackpack, Potion, WantsToDrinkPotion, CombatStats}};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUpItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            mut wants_pickup,
            mut positions,
            name,
            mut in_backpack
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            in_backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to insert item in backpack");

            if pickup.collected_by == *player_entity {
                game_log.entries.push(format!("You pick up the {}.", name.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct PotionSystem {}

impl<'a> System<'a> for PotionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut log,
            entities,
            mut wants_drink,
            names,
            potions,
            mut stats,
        ) = data;
        
        for (entity, drink, stats) in (&entities, &mut wants_drink, &mut stats).join() {
            let potion = potions.get(drink.item);
            match potion {
                None => {},
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if entity == *player_entity {
                        log.entries.push(
                            format!("You drink the {}, healing {} hp", names.get(drink.item).unwrap().name, potion.heal_amount)
                        );
                    }
                    entities.delete(drink.item).expect("Failed to delete potion entity");
                }
            }
        }

        wants_drink.clear();
    }
}