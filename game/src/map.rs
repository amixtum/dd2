use std::collections::HashSet;

use rltk::{Tile, RGB, Rect, Point, RandomNumberGenerator, Algorithm2D, BaseMap};
use specs::{World, WorldExt, Join};

use crate::{components::Viewshed, player::Player};

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: HashSet<Point>,
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
            if point.x < 80 && point.y < 40 {
                let idx = self.xy_flat(point.x, point.y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn draw_map(ecs: &World, ctx: &mut rltk::Rltk) {
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let mut players = ecs.write_storage::<Player>();
        let map = ecs.fetch::<Map>();

        for (_player, viewshed) in (&mut players, &mut viewsheds).join() {
            let mut x = 0;
            let mut y = 0;

            for tile in map.tiles.iter() {
                let point = Point::new(x, y);
                if viewshed.visible_tiles.contains(&point) {
                    match tile {
                        TileType::Floor => {
                            ctx.set(x, y, RGB::from_u8(127, 127, 127), RGB::from_u8(0, 0, 0), rltk::to_cp437('.'));
                        },
                        TileType::Wall => {
                            ctx.set(x, y, RGB::from_u8(0, 255, 0), RGB::from_u8(0, 0, 0), rltk::to_cp437('#'));
                        },
                    }
                }
                else if map.revealed_tiles.contains(&point) {
                    match tile {
                        TileType::Floor => {
                            ctx.set(x, y, RGB::from_u8(64, 64, 64), RGB::from_u8(0, 0, 0), rltk::to_cp437('.'));
                        },
                        TileType::Wall => {
                            ctx.set(x, y, RGB::from_u8(64, 64, 64), RGB::from_u8(0, 0, 0), rltk::to_cp437('#'));
                        },
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
}

impl Map {
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80*50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: HashSet::new(),
        };

        const MAX_ROOMS: i32 = 38;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
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
                        map.apply_vertical_tunnel( prev_center.x, prev_center.y, new_center.y);
                        map.apply_horizontal_tunnel(prev_center.x, new_center.x, new_center.y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        return Point::new(self.width, self.height);
    }
}