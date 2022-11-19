use std::collections::HashSet;

use rltk::{console, Algorithm2D, BaseMap, Point, PointF, RGB};
use specs::{Entity, Join, World, WorldExt};

use crate::{
    components::{Balance, CombatStats, Position, Velocity, Viewshed},
    movement_system::{MovementSystem, BALANCE_DAMP, FALLOVER, PLAYER_INST, SPEED_DAMP},
    player::Player,
};

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    //Rail {xdir: i32, ydir: i32},
}

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 50 - 6;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: HashSet<Point>,
    pub blocked_tiles: HashSet<Point>,
    pub tile_content: Vec<Vec<Entity>>,
    pub depth: i32,
    //pub render_table: HashMap<TileType, rltk::FontCharType>,
}

impl Map {
    pub fn xy_flat(&self, x: i32, y: i32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        !self.blocked_tiles.contains(&Point::new(x, y))
    }
}

pub fn cleanup_dead(ecs: &mut World) {
    let mut to_delete = Vec::new();
    {
        let combat_stats = ecs.write_storage::<CombatStats>();
        let positions = ecs.read_storage::<Position>();
        let ents = ecs.entities();
        let player_ent = ecs.fetch::<Entity>();
        for (ent, stats, pos) in (&ents, &combat_stats, &positions).join() {
            if stats.hp <= 0 {
                if ent == *player_ent {
                    console::log("Player is dead");
                } else {
                    to_delete.push((ent, pos.point));
                }
            }
        }
    }

    while to_delete.len() > 0 {
        if let Some((ent, pos)) = to_delete.pop() {
            ecs.delete_entity(ent).expect("Unable to delete entity");

            let mut map = ecs.fetch_mut::<Map>();
            map.blocked_tiles.remove(&pos);
        }
    }
}

fn get_simulation_color(
    map: &Map,
    speed: &Velocity,
    balance: &Balance,
    player_pos: &Point,
    map_pos: &Point,
) -> RGB {
    let inst_v = PointF::new(
        map_pos.x as f32 - player_pos.x as f32,
        map_pos.y as f32 - player_pos.y as f32,
    )
    .normalized()
        * PLAYER_INST;

    let sim_x = (player_pos.x as f32 + speed.vel.x * SPEED_DAMP + inst_v.x)
        .clamp(player_pos.x as f32 - 1.0, player_pos.x as f32 + 1.0)
        .round() as i32;
    let sim_y = (player_pos.y as f32 + speed.vel.y * SPEED_DAMP + inst_v.y)
        .clamp(player_pos.y as f32 - 1.0, player_pos.y as f32 + 1.0)
        .round() as i32;

    let balance = balance.bal * BALANCE_DAMP;
    let simulate_balance = MovementSystem::compute_balance(balance, speed.vel, inst_v);

    let fallover = simulate_balance.mag() / FALLOVER;
    let color: RGB;
    if fallover < 1.0 && !map.blocked_tiles.contains(&Point::new(sim_x, sim_y)) {
        color = RGB::from_f32(1.0 - fallover, 0.0, fallover);
    } else {
        color = RGB::from_f32(0.0, 1.0, 0.0);
    }

    color
}

impl Map {
    pub fn draw_map(map: &Map, ecs: &World, ctx: &mut rltk::Rltk) {
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let mut players = ecs.write_storage::<Player>();
        let balances = ecs.read_storage::<Balance>();
        let speeds = ecs.read_storage::<Velocity>();
        let player_pos = ecs.fetch::<Point>();

        for (_player, viewshed, balance, speed) in
            (&mut players, &mut viewsheds, &balances, &speeds).join()
        {
            let mut x = 0;
            let mut y = 0;

            for tile in map.tiles.iter() {
                let point = Point::new(x, y);
                if viewshed.visible_tiles.contains(&point) {
                    let color = get_simulation_color(&map, &speed, &balance, &player_pos, &point);
                    match tile {
                        TileType::Floor => {
                            ctx.set(x, y, color, RGB::from_u8(0, 0, 0), rltk::to_cp437('.'));
                        }
                        TileType::Wall => {
                            ctx.set(x, y, color, RGB::from_u8(0, 0, 0), rltk::to_cp437('#'));
                        }
                        TileType::DownStairs => {
                            ctx.set(x, y, color, RGB::from_u8(0, 0, 0), rltk::to_cp437('>'));
                        }
                    }
                } else if map.revealed_tiles.contains(&point) {
                    match tile {
                        TileType::Floor => {
                            ctx.set(
                                x,
                                y,
                                RGB::from_u8(64, 64, 64),
                                RGB::from_u8(0, 0, 0),
                                rltk::to_cp437('.'),
                            );
                        }
                        TileType::Wall => {
                            ctx.set(
                                x,
                                y,
                                RGB::from_u8(64, 64, 64),
                                RGB::from_u8(0, 0, 0),
                                rltk::to_cp437('#'),
                            );
                        }
                        TileType::DownStairs => {
                            ctx.set(
                                x,
                                y,
                                RGB::from_u8(64, 64, 64),
                                RGB::from_u8(0, 0, 0),
                                rltk::to_cp437('>'),
                            );
                        }
                    }
                }

                x += 1;
                if x >= map.width {
                    y += 1;
                    x = 0;
                }
            }
        }
    }

    pub fn populate_blocked(&mut self) {
        for tile in self.tiles.iter().enumerate() {
            if *tile.1 == TileType::Wall {
                let x = tile.0 as i32 % self.width as i32;
                let y = tile.0 as i32 / self.width as i32;
                self.blocked_tiles.insert(Point::new(x, y));
            }
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: HashSet::new(),
            blocked_tiles: HashSet::new(),
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth,
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0));
        }
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0));
        }
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0));
        }
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0));
        }

        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.0));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.0));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.0));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.0));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        return Point::new(self.width, self.height);
    }

    fn index_to_point2d(&self, idx: usize) -> Point {
        let w = self.width as usize;
        Point::new(idx % w, idx / w)
    }

    fn point2d_to_index(&self, pt: Point) -> usize {
        self.xy_flat(pt.x, pt.y)
    }

    fn in_bounds(&self, pos: Point) -> bool {
        pos.x > 0 && pos.y > 0 &&
        pos.x < self.width && pos.y < self.height
    }
}
