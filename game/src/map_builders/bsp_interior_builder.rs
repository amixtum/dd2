use rltk::{Point, RandomNumberGenerator, Rect};

use crate::{
    map::{Map, TileType, MAPHEIGHT, MAPWIDTH},
    spawner, SHOW_MAPGEN_VISUALIZER,
};

use super::MapBuilder;

const MIN_ROOM_SIZE: i32 = 5;

pub struct BspInteriorBuilder {
    map: Map,
    starting_position: Point,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspInteriorBuilder {
    fn build_map(&mut self, ecs: &mut specs::World) {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();

        self.rects.clear();
        self.rects.push(Rect {
            x1: 1,
            x2: self.map.width - 2,
            y1: 1,
            y2: self.map.height - 2,
        });
        let first_room = self.rects[0];

        // divide the first room
        self.add_subrects(first_room, &mut rng);

        let rooms = self.rects.clone(); 

        for r in rooms.iter() {
            let room = *r;
            self.rooms.push(room);

            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    let idx = self.map.xy_flat(x, y);
                    if idx > 0 && idx < ((self.map.width * self.map.height) - 1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            self.take_snapshot();
        }

        self.rooms.sort_by(|a, b| {
            a.x1.cmp(&b.x1)
        });

        for i in 0..self.rooms.len()-1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i + 1];

            let start_x = room.x1 + rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1;
            let start_y = room.y1 + rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1;
            let end_x = next_room.x1 + rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1;
            let end_y = next_room.y1 + rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1;
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
        }

        self.starting_position = self.rooms[0].center();

        let stairs_position = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_flat(stairs_position.x, stairs_position.y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Point {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for y in 0..MAPHEIGHT {
                for x in 0..MAPWIDTH {
                    snapshot.revealed_tiles.insert(Point::new(x, y));
                }
            }
            self.history.push(snapshot);
        }
    }
}

impl BspInteriorBuilder {
    pub fn new(new_depth: i32) -> Self {
        BspInteriorBuilder {
            map: Map::new(new_depth),
            starting_position: Point::new(0, 0),
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new(),
        }
    }
}

impl BspInteriorBuilder {
    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        if !self.rects.is_empty() {
            self.rects.pop();
        }

        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = width / 2;
        let half_height = height / 2;

        let split = rng.roll_dice(1, 100);

        if split <= 50 {
            // split horizontal
            let h1 = Rect {
                x1: rect.x1,
                x2: rect.x1 + half_width - 1,
                y1: rect.y1,
                y2: rect.y2,
            };
            self.rects.push(h1);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h1, rng);
            } 

            let h2 = Rect {
                x1: rect.x1 + half_width,
                x2: rect.x2,
                y1: rect.y1,
                y2: rect.y2,
            };
            self.rects.push(h2);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h2, rng);
            } 
        }
        else {
            // split vertical 
            let v1 = Rect {
                x1: rect.x1,
                x2: rect.x2,
                y1: rect.y1,
                y2: rect.y1 + half_height - 1,
            };
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v1, rng);
            } 

            let v2 = Rect {
                x1: rect.x1,
                x2: rect.x2,
                y1: rect.y1 + half_height,
                y2: rect.y2,
            };
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v2, rng);
            } 
        }
    }

    fn draw_corridor(&mut self, start_x: i32, start_y: i32, end_x: i32, end_y: i32) {
        let mut x = start_x;
        let mut y = start_y;

        while x != end_x || y != end_y {
            if x < end_x {
                x += 1;
            }
            else if x > end_x {
                x -= 1;
            }
            else if y < end_y {
                y += 1;
            }
            else if y > end_y {
                y -= 1;
            }

            let idx = self.map.xy_flat(x, y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}