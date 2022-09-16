use rltk::{Point, PointF, RandomNumberGenerator};
use specs::prelude::*;
use util::vec_ops;

use crate::{
    components::{
        Balance, BlocksTile, CombatStats, InstVel, Monster, Position, Speed, WantsToFallover,
        WantsToMelee,
    },
    map::{self, Map},
};

pub const MAX_SPEED: f32 = 3.0;
pub const SPEED_DAMP: f32 = 0.66;

pub const ZERO_SPEED: f32 = 0.25;

pub const BALANCE_DAMP: f32 = 0.5;
pub const LEAN_FACTOR: f32 = 0.66;
pub const FALLOVER: f32 = 1.0;

pub struct SpeedBalanceSystem {}

impl<'a> System<'a> for SpeedBalanceSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, InstVel>,
        WriteStorage<'a, Balance>,
        WriteStorage<'a, WantsToFallover>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut speeds, mut inst_vels, mut balances, mut fallovers) = data;

        for (entity, speed, balance) in (&entities, &mut speeds, &mut balances).join() {
            // apply dampening
            speed.speed *= SPEED_DAMP;
            balance.bal *= BALANCE_DAMP;

            // add inst velocities to speed
            if let Some(inst_v) = inst_vels.get_mut(entity) {
                let mut instv_sum: PointF = PointF::new(0.0, 0.0);

                for vel in inst_v.vel.iter() {
                    instv_sum += *vel;
                }

                let last_speed = speed.speed;

                speed.speed += instv_sum;

                let mag = speed.speed.mag();
                if mag > MAX_SPEED {
                    speed.speed *= MAX_SPEED / speed.speed.mag();
                } else if mag <= ZERO_SPEED {
                    speed.speed = PointF::new(0.0, 0.0);
                }

                // entity leans in direction they were last moving and are not moving anymore
                if last_speed.mag() > ZERO_SPEED {
                    let direction_diff = last_speed - instv_sum;
                    let orthogonality = (last_speed.mag() * instv_sum.mag()
                        - last_speed.dot(instv_sum))
                        / last_speed.mag()
                        * instv_sum.mag();

                    balance.bal.x += direction_diff.y.signum() * orthogonality * LEAN_FACTOR;
                    balance.bal.y += direction_diff.x.signum() * orthogonality * LEAN_FACTOR;
                }
                // entity leans in direction they move if their speed is below ZERO_SPEED
                // and their inst velocity is greater than ZERO_SPEED
                else {
                    if instv_sum.mag() > ZERO_SPEED {
                        balance.bal.x += instv_sum.normalized().x * LEAN_FACTOR;
                        balance.bal.y += instv_sum.normalized().y * LEAN_FACTOR;
                    }
                }
            }

            if balance.bal.mag() >= FALLOVER {
                speed.speed = PointF::new(0.0, 0.0);
                balance.bal = PointF::new(0.0, 0.0);
                fallovers
                    .insert(entity, WantsToFallover {})
                    .expect("Unable to insert intent to fallover");
            }
        }

        inst_vels.clear();
    }
}

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Speed>,
        WriteStorage<'a, WantsToFallover>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut rng,
            mut positions,
            speeds,
            mut fallovers,
            mut wants_melee,
            combat_stats,
            monsters,
        ) = data;

        for (entity, pos, speed) in (&entities, &mut positions, &speeds).join() {
            let mut x = (pos.point.x as f32 + speed.speed.x)
                .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
                .round() as i32;
            let mut y = (pos.point.y as f32 + speed.speed.y)
                .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
                .round() as i32;
            let mut next = Point::new(x, y);

            // check if we run into a wall or the edge of the map
            // and insert a fallover intent
            // if we run into a monster
            if x < 0 || x > map.width - 1 || y < 0 || y > map.height - 1 {
                fallovers
                    .insert(entity, WantsToFallover {})
                    .expect("Could not insert intent to fallover");
                return;
            } else if map.blocked_tiles.contains(&next) {
                let dest_idx = map.xy_flat(x, y);
                for potential_target in map.tile_content[dest_idx].iter() {
                    if let Some(other_speed) = speeds.get(*potential_target) {
                        let mut new_speed = speed.speed + other_speed.speed;
                        x = (pos.point.x as f32 + new_speed.x)
                            .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
                            .round() as i32;
                        y = (pos.point.y as f32 + new_speed.y)
                            .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
                            .round() as i32;
                        next = Point::new(x, y);

                        // TODO
                        // check that we are not colliding anymore
                        while map.blocked_tiles.contains(&next) {
                            let units = vec_ops::discrete_jmp((new_speed.x, new_speed.y));
                            let neighbors = vec_ops::neighbors(
                                (next.x, next.y),
                                (1, 1),
                                (map.width - 2, map.height - 2),
                            )
                            .iter()
                            .filter(|p| match units.0.signum() {
                                -1 => match units.1.signum() {
                                    -1 => p.0 == next.x - 1 || p.1 == next.y - 1,
                                    0 => p.0 == next.x - 1,
                                    1 => p.0 == next.x - 1 || p.1 == next.y + 1,
                                    _ => false,
                                },
                                0 => match units.1.signum() {
                                    -1 => p.1 == next.y - 1,
                                    0 => true,
                                    1 => p.1 == next.y + 1,
                                    _ => false,
                                },
                                1 => match units.1.signum() {
                                    -1 => p.0 == next.x + 1 || p.1 == next.y - 1,
                                    0 => p.0 == next.x + 1,
                                    1 => p.0 == next.x + 1 || p.1 == next.y + 1,
                                    _ => false,
                                },
                                _ => false,
                            })
                            .collect::<Vec<_>>();
                        }
                        // HERE

                        if x < 0 || x > map.width - 1 || y < 0 || y > map.height - 1 {
                            fallovers
                                .insert(entity, WantsToFallover {})
                                .expect("Could not insert intent to fallover");
                            return;
                        }
                    }
                }
            }

            // update position
            map.blocked_tiles.remove(&pos.point);
            pos.point = next;
            map.blocked_tiles.insert(pos.point);
        }
    }
}
