use rltk::{DistanceAlg, Point, PointF, RandomNumberGenerator, console};
use specs::prelude::*;
use util::vec_ops::{self};

use crate::{
    components::{Balance, InstVel, Position, Speed, WantsToFallover, CombatStats},
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
pub const FALLOVER: f32 = 5.0;

pub struct FalloverSystem {}

impl<'a> System<'a> for FalloverSystem {
    type SystemData = (
        WriteStorage<'a, WantsToFallover>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, Balance>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let ( mut fallovers, mut speeds, mut balances, mut combat_stats) = data;

        for ( _fall, speed, balance, _stats) in (&mut fallovers, &mut speeds, &mut balances, &mut combat_stats).join() {
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

            // add inst velocities to speed
            if let Some(inst_v) = inst_vels.get_mut(entity) {
                let mut instv_sum: PointF = PointF::new(0.0, 0.0);

                for vel in inst_v.vel.iter() {
                    instv_sum.x += vel.x;
                    instv_sum.y += vel.y;
                }

                let last_speed = speed.speed;

                speed.speed.x += instv_sum.x;
                speed.speed.y += instv_sum.y;

                console::log(format!("speed = ({}, {})", speed.speed.x, speed.speed.y));

                let mag = speed.speed.mag();
                if mag > MAX_SPEED {
                    speed.speed *= MAX_SPEED / speed.speed.mag();
                } else if mag <= ZERO_SPEED {
                    speed.speed = PointF::new(0.0, 0.0);
                }

                // entity leans in direction they were last moving and are not moving anymore
                if last_speed.mag() > ZERO_SPEED {
                    let direction_diff = last_speed - instv_sum;
                    let units = vec_ops::discrete_jmp((direction_diff.x, direction_diff.y));
                    let orthogonality = (2.0 * last_speed.mag() * instv_sum.mag()
                        - last_speed.dot(instv_sum))
                        / (2.0 * last_speed.mag()
                        * instv_sum.mag());

                    balance.bal.x += units.1.signum() as f32 * orthogonality * LEAN_FACTOR;
                    balance.bal.y += units.0.signum() as f32 * orthogonality * LEAN_FACTOR;
                }
            }

            if balance.bal.mag() >= FALLOVER {
                fallovers
                    .insert(entity, WantsToFallover {})
                    .expect("Unable to insert intent to fallover");
            }
            else if balance.bal.mag() <= ZERO_BALANCE {
                balance.bal = PointF::new(0.0, 0.0);
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
        WriteExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Speed>,
        WriteStorage<'a, WantsToFallover>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, mut _rng, mut player_pos, player_entity, mut positions, speeds, mut fallovers) = data;

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
                console::log(format!("next = ({}, {})", next.x, next.y));
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

fn _compute_slide(
    map: &Map,
    pos: &Position,
    speed: &Speed,
    other_speed: &Speed,
    rng: &mut RandomNumberGenerator,
) -> Option<Point> {
    let new_speed = speed.speed + other_speed.speed;
    let x = (pos.point.x as f32 + new_speed.x)
        .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
        .clamp(0.0, map.width as f32 - 1.0)
        .round() as i32;
    let y = (pos.point.y as f32 + new_speed.y)
        .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
        .clamp(0.0, map.height as f32 - 1.0)
        .round() as i32;
    let mut next = Point::new(x, y);

    // TODO
    // check that we are not colliding with anything after 'sliding'
    let mut blocked = map.blocked_tiles.contains(&next);
    if blocked {
        let units = vec_ops::discrete_jmp((new_speed.x, new_speed.y));
        let nbrs = vec_ops::neighbors((next.x, next.y), (1, 1), (map.width - 2, map.height - 2));
        let mut nbrs_can_bounce = nbrs
            .iter()
            .filter(|p| match units.0.signum() {
                -1 => match units.1.signum() {
                    -1 => {
                        (p.0 == next.x - 1 || p.0 == next.x) && (p.1 == next.y - 1 || p.1 == next.y)
                    }
                    0 => p.0 == next.x - 1,
                    1 => {
                        (p.0 == next.x - 1 || p.0 == next.x) && (p.1 == next.y + 1 || p.1 == next.y)
                    }
                    _ => false,
                },
                0 => match units.1.signum() {
                    -1 => p.1 == next.y - 1,
                    0 => true,
                    1 => p.1 == next.y + 1,
                    _ => false,
                },
                1 => match units.1.signum() {
                    -1 => {
                        (p.0 == next.x + 1 || p.0 == next.x) && (p.1 == next.y - 1 || p.1 == next.y)
                    }
                    0 => p.0 == next.x + 1,
                    1 => {
                        (p.0 == next.x + 1 || p.0 == next.x) && (p.1 == next.y + 1 || p.1 == next.y)
                    }
                    _ => false,
                },
                _ => false,
            })
            .filter(|p| !map.blocked_tiles.contains(&Point::new(p.0, p.1)))
            .collect::<Vec<_>>();

        // we cannot slide one tile to an unblocked position
        // so we set blocked to true whichmakes us fall over
        if nbrs_can_bounce.len() < 1 {
            blocked = true;
        } else if nbrs_can_bounce.len() == 1 {
            next = Point::new(nbrs_can_bounce[0].0, nbrs_can_bounce[0].1);
            blocked = false;
        } else {
            // sort by decreasing distance from point obtained by adding our new speed (after slide)
            nbrs_can_bounce.sort_by(|l, r| {
                let ldist = DistanceAlg::Pythagoras
                    .distance2d(Point::new(l.0, l.1), next)
                    .round() as i32;
                let rdist = DistanceAlg::Pythagoras
                    .distance2d(Point::new(r.0, r.1), next)
                    .round() as i32;
                ldist.cmp(&rdist)
            });

            // weighted choice
            let mut weights = Vec::new();
            let mut sum = 0;
            for n in 0..nbrs_can_bounce.len() {
                let sqr = (nbrs_can_bounce.len() - n).pow(2);
                sum += sqr as i32;
                weights.push(sqr as i32);
            }
            while blocked {
                for weight in weights.iter().enumerate() {
                    let roll = rng.roll_dice(1, sum);
                    if roll <= *weight.1 {
                        let choice = nbrs_can_bounce[weight.0 as usize];
                        next = Point::new(choice.0, choice.1);
                        blocked = false;
                        break;
                    }
                }
            }
        }
    }
    if blocked {
        return None;
    } else {
        return Some(next);
    }
}

fn _compute_wall_slide(
    map: &Map,
    pos: &Position,
    speed: &Speed,
    rng: &mut RandomNumberGenerator,
) -> Option<Point> {
    let new_speed = speed.speed;
    let x = (pos.point.x as f32 + new_speed.x)
        .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
        .round() as i32;
    let y = (pos.point.y as f32 + new_speed.y)
        .clamp(pos.point.x as f32 - 1.0, pos.point.x as f32 + 1.0)
        .round() as i32;
    let mut next = Point::new(x, y);

    // TODO
    // check that we are not colliding with anything after 'sliding'
    let mut blocked = map.blocked_tiles.contains(&next);
    if blocked {
        let units = vec_ops::discrete_jmp((new_speed.x, new_speed.y));
        let nbrs = vec_ops::neighbors((next.x, next.y), (1, 1), (map.width - 2, map.height - 2));
        let mut nbrs_can_bounce = nbrs
            .iter()
            .filter(|p| match units.0.signum() {
                -1 => match units.1.signum() {
                    -1 => {
                        (p.0 == next.x - 1 || p.0 == next.x) && (p.1 == next.y - 1 || p.1 == next.y)
                    }
                    0 => p.0 == next.x - 1,
                    1 => {
                        (p.0 == next.x - 1 || p.0 == next.x) && (p.1 == next.y + 1 || p.1 == next.y)
                    }
                    _ => false,
                },
                0 => match units.1.signum() {
                    -1 => p.1 == next.y - 1,
                    0 => true,
                    1 => p.1 == next.y + 1,
                    _ => false,
                },
                1 => match units.1.signum() {
                    -1 => {
                        (p.0 == next.x + 1 || p.0 == next.x) && (p.1 == next.y - 1 || p.1 == next.y)
                    }
                    0 => p.0 == next.x + 1,
                    1 => {
                        (p.0 == next.x + 1 || p.0 == next.x) && (p.1 == next.y + 1 || p.1 == next.y)
                    }
                    _ => false,
                },
                _ => false,
            })
            .filter(|p| !map.blocked_tiles.contains(&Point::new(p.0, p.1)))
            .collect::<Vec<_>>();

        // we cannot slide one tile to an unblocked position
        // so we set blocked to true whichmakes us fall over
        if nbrs_can_bounce.len() < 1 {
            blocked = true;
        } else if nbrs_can_bounce.len() == 1 {
            next = Point::new(nbrs_can_bounce[0].0, nbrs_can_bounce[0].1);
            blocked = false;
        } else {
            // sort by decreasing distance from point obtained by adding our new speed (after slide)
            nbrs_can_bounce.sort_by(|l, r| {
                let ldist = DistanceAlg::Pythagoras
                    .distance2d(Point::new(l.0, l.1), next)
                    .round() as i32;
                let rdist = DistanceAlg::Pythagoras
                    .distance2d(Point::new(r.0, r.1), next)
                    .round() as i32;
                ldist.cmp(&rdist)
            });

            // weighted choice
            let mut weights = Vec::new();
            let mut sum = 0;
            for n in 0..nbrs_can_bounce.len() {
                let sqr = (nbrs_can_bounce.len() - n).pow(2);
                sum += sqr as i32;
                weights.push(sqr as i32);
            }
            while blocked {
                for weight in weights.iter().enumerate() {
                    let roll = rng.roll_dice(1, sum);
                    if roll <= *weight.1 {
                        let choice = nbrs_can_bounce[weight.0 as usize];
                        next = Point::new(choice.0, choice.1);
                        blocked = false;
                        break;
                    }
                }
            }
        }
    }
    if blocked {
        return None;
    } else {
        return Some(next);
    }
}
