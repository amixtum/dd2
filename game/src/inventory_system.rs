use specs::prelude::*;

use crate::{gamelog::GameLog, components::{WantsToPickUpItem, Position, Name, InBackpack, ProvidesHealing, WantsToUseItem, CombatStats, Consumable, InflictsDamage, SufferDamage, AreaOfEffect}, map::Map};

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

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            player_entity,
            mut log,
            entities,
            mut use_item_intents,
            names,
            consumables,
            healing,
            mut stats,
            inflict_damage,
            mut suffer_damage,
            aoe,
        ) = data;
        
        for (entity, use_item, mut stats) in (&entities, &use_item_intents, &mut stats).join() {
            if let Some(_) = consumables.get(use_item.item) { 
                if let Some(healing) = healing.get(use_item.item) {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                    if entity == *player_entity {
                        log.entries.push(
                            format!("You drink the {}, healing {} hp", names.get(use_item.item).unwrap().name, healing.heal_amount)
                        );
                    }
                } else if let Some(damage) = inflict_damage.get(use_item.item) {
                    if let Some(target_pos) = use_item.target {
                        let mut targets = Vec::new();
                        if let Some(aoe) = aoe.get(use_item.item) {
                            let mut blast_tiles = rltk::field_of_view(target_pos, aoe.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });

                            for tile in blast_tiles.iter() {
                                let idx = map.xy_flat(tile.x, tile.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                        else {
                            let idx = map.xy_flat(target_pos.x, target_pos.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);

                            }

                        }
                        for mob in targets.iter() {
                            SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                            if entity == *player_entity {
                                let mob_name = &names.get(*mob).unwrap().name;
                                let item_name = &names.get(use_item.item).unwrap().name;
                                log.entries.push(format!("You use {} on {}, inflicting {} damage.", item_name, mob_name, damage.damage));
                            }
                        }

                    }


                }

                entities.delete(use_item.item).expect("Deleting consumable failed");
            }
        }

        use_item_intents.clear();
    }
}