use rltk::{Point, PointF, RandomNumberGenerator};
use specs::prelude::*;
use util::vec_ops::{self};

use crate::{
    components::{Balance, CombatStats, InstVel, Position, Velocity, WantsToFallover},
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
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Balance>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut fallovers, mut vels, mut balances, mut combat_stats) = data;

        for (_fall, vel, balance, _stats) in (
            &mut fallovers,
            &mut vels,
            &mut balances,
            &mut combat_stats,
        )
            .join()
        {
            vel.vel = PointF::new(0.0, 0.0);
            balance.bal = PointF::new(0.0, 0.0);
            //stats.hp = std::cmp::max(0, stats.hp - 1);
        }

        fallovers.clear();
    }
}

pub struct VelocityBalanceSystem {}

impl<'a> System<'a> for VelocityBalanceSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, InstVel>,
        WriteStorage<'a, Balance>,
        WriteStorage<'a, WantsToFallover>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut vels, mut inst_vels, mut balances, mut fallovers) = data;

        for (entity, velocity, balance) in (&entities, &mut vels, &mut balances).join() {
            // apply dampening
            velocity.vel *= SPEED_DAMP;
            balance.bal *= BALANCE_DAMP;

            if let Some(inst_v) = inst_vels.get_mut(entity) {
                let last_vel = velocity.vel;

                let mut instv_sum: PointF = PointF::new(0.0, 0.0);

                for vel in inst_v.vel.iter() {
                    instv_sum.x += vel.x;
                    instv_sum.y += vel.y;
                }

                // add sum of inst velocities to speed
                velocity.vel = MovementSystem::compute_vel_cached_sum(velocity.vel, instv_sum);

                //console::log(format!("speed = ({}, {})", speed.speed.x, speed.speed.y));

                // compute orthogonal movement's contribution to balance
                balance.bal = MovementSystem::compute_balance(balance.bal, last_vel, instv_sum);
            }

            let mag = velocity.vel.mag();

            // clamp to max_speed
            if mag > MAX_SPEED {
                velocity.vel *= MAX_SPEED / velocity.vel.mag();

            // zero speed below this threshold
            } else if mag <= ZERO_SPEED {
                velocity.vel = PointF::new(0.0, 0.0);
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
    pub fn compute_vel_cached_sum(vel: PointF, inst_vel: PointF) -> PointF {
        vel + inst_vel
    }

    pub fn compute_balance(balance: PointF, last_vel: PointF, inst_vel: PointF) -> PointF {
        let mut balance = balance;
        // entity leans in direction they were last moving and are not moving anymore
        if last_vel.mag() > ZERO_SPEED && inst_vel.mag() > 0.01 {
            let direction_diff = last_vel - inst_vel;
            let units = vec_ops::discrete_jmp((direction_diff.x, direction_diff.y));
            let orthogonality = (2.0 * last_vel.mag() * inst_vel.mag()
                - last_vel.dot(inst_vel))
                / (2.0 * last_vel.mag() * inst_vel.mag());

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
        ReadStorage<'a, Velocity>,
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
            vels,
            mut fallovers,
        ) = data;

        let mut sort_by_vel = (&entities, &mut positions, &vels)
            .join()
            .collect::<Vec<_>>();
        sort_by_vel.sort_by(|l, r| {
            (l.2.vel.mag().round() as i32).cmp(&(r.2.vel.mag().round() as i32))
        });
        for (entity, pos, vel) in sort_by_vel.iter_mut().rev() {
            let x = (pos.point.x as f32 + vel.vel.x)
                .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
                .round() as i32;
            let y = (pos.point.y as f32 + vel.vel.y)
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
