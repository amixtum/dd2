use rltk::{Point, PointF, RandomNumberGenerator};
use specs::prelude::*;
use util::vec_ops::{self};

use crate::{
    components::{Balance, CombatStats, InstVel, Position, Speed, WantsToFallover},
    map::Map,
};

pub const PLAYER_INST: f32 = 0.77;
pub const MONSTER_INST: f32 = 0.66;

pub const MAX_SPEED: f32 = 3.0;
pub const SPEED_DAMP: f32 = 0.66;

pub const ZERO_SPEED: f32 = 0.5;
pub const ZERO_BALANCE: f32 = 0.25;

pub const BALANCE_DAMP: f32 = 0.5;
pub const LEAN_FACTOR: f32 = 0.66;
pub const FALLOVER: f32 = 1.33;

pub struct FalloverSystem {}

impl<'a> System<'a> for FalloverSystem {
    type SystemData = (
        WriteStorage<'a, WantsToFallover>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, Balance>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut fallovers, mut speeds, mut balances, mut combat_stats) = data;

        for (_fall, speed, balance, _stats) in (
            &mut fallovers,
            &mut speeds,
            &mut balances,
            &mut combat_stats,
        )
            .join()
        {
            speed.speed = PointF::new(0.0, 0.0);
            balance.bal = PointF::new(0.0, 0.0);
            //stats.hp = std::cmp::max(0, stats.hp - 1);
        }

        fallovers.clear();
    }
}

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

            if let Some(inst_v) = inst_vels.get_mut(entity) {
                let last_speed = speed.speed;

                let mut instv_sum: PointF = PointF::new(0.0, 0.0);

                for vel in inst_v.vel.iter() {
                    instv_sum.x += vel.x;
                    instv_sum.y += vel.y;
                }

                // add sum of inst velocities to speed
                speed.speed = MovementSystem::compute_speed_cached_sum(speed.speed, instv_sum);

                //console::log(format!("speed = ({}, {})", speed.speed.x, speed.speed.y));

                // compute orthogonal movement's contribution to balance
                balance.bal = MovementSystem::compute_balance(balance.bal, last_speed, instv_sum);
            }

            let mag = speed.speed.mag();

            // clamp to max_speed
            if mag > MAX_SPEED {
                speed.speed *= MAX_SPEED / speed.speed.mag();

            // zero speed below this threshold
            } else if mag <= ZERO_SPEED {
                speed.speed = PointF::new(0.0, 0.0);
            }

            // fallover when balance is too large
            if balance.bal.mag() >= FALLOVER {
                fallovers
                    .insert(entity, WantsToFallover {})
                    .expect("Unable to insert intent to fallover");
            }
            // zero balance below this threshold
            else if balance.bal.mag() <= ZERO_BALANCE {
                balance.bal = PointF::new(0.0, 0.0);
            }
        }

        inst_vels.clear();
    }
}

impl MovementSystem {
    pub fn compute_speed_component(speed: PointF, inst_vel: &InstVel) -> PointF {
        let mut speed = speed * SPEED_DAMP;

        let mut instv_sum: PointF = PointF::new(0.0, 0.0);

        for vel in inst_vel.vel.iter() {
            instv_sum.x += vel.x;
            instv_sum.y += vel.y;
        }

        speed.x += instv_sum.x;
        speed.y += instv_sum.y;

        speed
    }

    pub fn compute_speed_cached_sum(speed: PointF, inst_vel: PointF) -> PointF {
        speed + inst_vel
    }

    pub fn compute_balance(balance: PointF, last_speed: PointF, inst_vel: PointF) -> PointF {
        let mut balance = balance;
        // entity leans in direction they were last moving and are not moving anymore
        if last_speed.mag() > ZERO_SPEED && inst_vel.mag() > 0.01 {
            let direction_diff = last_speed - inst_vel;
            let units = vec_ops::discrete_jmp((direction_diff.x, direction_diff.y));
            let orthogonality = (2.0 * last_speed.mag() * inst_vel.mag()
                - last_speed.dot(inst_vel))
                / (2.0 * last_speed.mag() * inst_vel.mag());

            balance.x += units.1.signum() as f32 * orthogonality * LEAN_FACTOR;
            balance.y += units.0.signum() as f32 * orthogonality * LEAN_FACTOR;
        }

        balance
    }
}

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Speed>,
        WriteStorage<'a, WantsToFallover>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut map,
            mut _rng,
            mut player_pos,
            player_entity,
            mut positions,
            speeds,
            mut fallovers,
        ) = data;

        let mut sort_by_speed = (&entities, &mut positions, &speeds)
            .join()
            .collect::<Vec<_>>();
        sort_by_speed.sort_by(|l, r| {
            (l.2.speed.mag().round() as i32).cmp(&(r.2.speed.mag().round() as i32))
        });
        for (entity, pos, speed) in sort_by_speed.iter_mut().rev() {
            let x = (pos.point.x as f32 + speed.speed.x)
                .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
                .round() as i32;
            let y = (pos.point.y as f32 + speed.speed.y)
                .clamp(pos.point.y as f32 - 1.0, pos.point.y as f32 + 1.0)
                .round() as i32;

            // nothing to update
            if x == pos.point.x && y == pos.point.y {
                continue;
            }

            let next = Point::new(x, y);
            let mut blocked = false;

            // check if we run run over the edge of the map
            // insert a fallover intent and return
            if x < 0 || x > map.width - 1 || y < 0 || y > map.height - 1 {
                fallovers
                    .insert(*entity, WantsToFallover {})
                    .expect("Could not insert intent to fallover");
                return;
            // we encounter a blocked tile
            } else if map.blocked_tiles.contains(&next) && next != pos.point {
                blocked = true;
            }

            // fallover if we are off the map or still blocked
            if x < 0 || x > map.width - 1 || y < 0 || y > map.height - 1 || blocked {
                fallovers
                    .insert(*entity, WantsToFallover {})
                    .expect("Could not insert intent to fallover");
            }
            // update position
            else {
                //console::log(format!("next = ({}, {})", next.x, next.y));
                map.blocked_tiles.remove(&pos.point);
                pos.point = next;

                if *entity == *player_entity {
                    player_pos.x = next.x;
                    player_pos.y = next.y;
                }

                map.blocked_tiles.insert(pos.point);
            }
        }
    }
}
