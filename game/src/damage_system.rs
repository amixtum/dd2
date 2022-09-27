use specs::prelude::*;

use crate::{gamelog::GameLog, components::{SufferDamage, CombatStats, Name}};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_ent,
            mut log,
            ents,
            names,
            mut damages,
            mut combat_stats, 
        ) = data;

        for (ent, name, dmg, stats) in (&ents, &names, &mut damages, &mut combat_stats).join() {
            let mut sum_dmg = 0;
            for d in dmg.amount.iter() {
                sum_dmg += d;
            }
            stats.hp = std::cmp::max(0, stats.hp - sum_dmg);

            if ent == *player_ent {
                log.entries.push(format!("You take {} damage", sum_dmg));
            }
            else {
                log.entries.push(format!("{} takes {} damage, leaving them with {} hp", name.name, sum_dmg, stats.hp));
            }
        }

        damages.clear();
    }
}