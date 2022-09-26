use std::collections::{HashSet};

use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rect, RGB, PointF};
use specs::{Entity, Join, World, WorldExt};

use crate::{components::{Viewshed, Balance, Speed}, player::{Player}, movement_system::{MovementSystem, PLAYER_INST, FALLOVER, BALANCE_DAMP, SPEED_DAMP}};

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Wall,
    Floor,
    //Rail {xdir: i32, ydir: i32},
}

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 50 - 6;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: HashSet<Point>,
    pub blocked_tiles: HashSet<Point>,
    pub tile_content: Vec<Vec<Entity>>,
    //pub render_table: HashMap<TileType, rltk::FontCharType>,
}

impl Map {
    pub fn xy_flat(&self, x: i32, y: i32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    fn apply_room_to_map(&mut self, room: Rect) {
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = self.xy_flat(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        self.apply_tunnel(x1, y, x2, y);
    }

    fn apply_vertical_tunnel(&mut self, x: i32, y1: i32, y2: i32) {
        self.apply_tunnel(x, y1, x, y2);
    }

    fn apply_tunnel(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        for point in rltk::line2d_bresenham(Point::new(x1, y1), Point::new(x2, y2)) {
            if point.x < MAPWIDTH as i32 && point.y < MAPHEIGHT as i32 {
                let idx = self.xy_flat(point.x, point.y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        !self.blocked_tiles.contains(&Point::new(x, y))
    }
}

fn get_simulation_color(map: &Map, speed: &Speed, balance: &Balance, player_pos: &Point, map_pos: &Point) -> RGB {
    let inst_v = PointF::new(map_pos.x as f32 - player_pos.x as f32, map_pos.y as f32 - player_pos.y as f32).normalized() * PLAYER_INST;

    let sim_x = (player_pos.x as f32 + speed.speed.x * SPEED_DAMP + inst_v.x)
        .clamp(player_pos.x as f32 - 1.0, player_pos.x as f32 + 1.0)
        .round() as i32;
    let sim_y = (player_pos.y as f32 + speed.speed.y * SPEED_DAMP + inst_v.y)
        .clamp(player_pos.y as f32 - 1.0, player_pos.y as f32 + 1.0)
        .round() as i32;

    let balance = balance.bal * BALANCE_DAMP;
    let simulate_balance = MovementSystem::compute_balance(balance, speed.speed, inst_v);

    let fallover = simulate_balance.mag() / FALLOVER;
    let color: RGB;
    if fallover < 1.0 && !map.blocked_tiles.contains(&Point::new(sim_x, sim_y)) {
        color = RGB::from_f32(1.0 - fallover, 0.0, fallover);
    }
    else {
        color = RGB::from_f32(0.0, 1.0, 0.0);
    }

    color
}

impl Map {
    pub fn draw_map(ecs: &World, ctx: &mut rltk::Rltk) {
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let mut players = ecs.write_storage::<Player>();
        let balances = ecs.read_storage::<Balance>();
        let speeds = ecs.read_storage::<Speed>();
        let player_pos = ecs.fetch::<Point>();
        let map = ecs.fetch::<Map>();

        for (_player, viewshed, balance, speed) in (&mut players, &mut viewsheds, &balances, &speeds).join() {
            let mut x = 0;
            let mut y = 0;

            for tile in map.tiles.iter() {
                let point = Point::new(x, y);
                if viewshed.visible_tiles.contains(&point) {
                    let color = get_simulation_color(&map, &speed, &balance, &player_pos, &point);
                    match tile {
                        TileType::Floor => {
                            ctx.set(
                                x,
                                y,
                                color,
                                RGB::from_u8(0, 0, 0),
                                rltk::to_cp437('.'),
                            );
                        }
                        TileType::Wall => {
                            ctx.set(
                                x,
                                y,
                                color,
                                RGB::from_u8(0, 0, 0),
                                rltk::to_cp437('#'),
                            );
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

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: HashSet::new(),
            blocked_tiles: HashSet::new(),
            tile_content: vec![Vec::new(); MAPCOUNT],
        };

        const MAX_ROOMS: i32 = 38;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, MAPWIDTH as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, MAPHEIGHT as i32 - h - 1) - 1;
            let new_room = Rect {
                x1: x,
                y1: y,
                x2: x + w,
                y2: y + h,
            };
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
            if ok {
                map.apply_room_to_map(new_room);

                if !map.rooms.is_empty() {
                    let new_center = new_room.center();
                    let prev_center = map.rooms[map.rooms.len() - 1].center();

                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_center.x, new_center.x, prev_center.y);
                        map.apply_vertical_tunnel(new_center.x, prev_center.y, new_center.y);
                    } else {
                        map.apply_vertical_tunnel(prev_center.x, prev_center.y, new_center.y);
                        map.apply_horizontal_tunnel(prev_center.x, new_center.x, new_center.y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map.populate_blocked();

        map
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
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
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
}
