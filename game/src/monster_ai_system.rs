use specs::prelude::*;

use crate::{
    components::{Position, WantsToMelee},
    map::Map,
    state::RunState,
};

use super::{Monster, Viewshed};
use rltk::Point;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            viewshed,
            monster,
            mut position,
            mut wants_to_melee,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _monster, mut pos) in
            (&entities, &viewshed, &monster, &mut position).join()
        {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(pos.point, *player_pos);
            if distance < 1.5 {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *player_entity,
                        },
                    )
                    .expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&player_pos) {
                let path = rltk::a_star_search(
                    map.xy_flat(pos.point.x, pos.point.y) as i32,
                    map.xy_flat(player_pos.x, player_pos.y) as i32,
                    &mut *map,
                );

                if path.success && path.steps.len() > 1 {
                    map.blocked_tiles.remove(&pos.point);

                    pos.point.x = path.steps[1] as i32 % map.width;
                    pos.point.y = path.steps[1] as i32 / map.width;

                    map.blocked_tiles.insert(pos.point);
                }
            }
        }
    }
}
