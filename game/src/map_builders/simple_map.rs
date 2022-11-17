use crate::map::TileType;
use crate::map::MAPHEIGHT;
use crate::map::MAPWIDTH;
use crate::map_builders::common::apply_horizontal_tunnel;
use crate::map_builders::common::apply_room_to_map;
use crate::map_builders::common::apply_vertical_tunnel;
use crate::spawner;
use crate::SHOW_MAPGEN_VISUALIZER;

use super::Map;
use super::MapBuilder;
use rltk::Point;
use rltk::RandomNumberGenerator;
use rltk::Rect;
use specs::prelude::*;

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Point,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, ecs: &mut World) {
        self.rooms_and_corridors(ecs);
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
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

impl SimpleMapBuilder {
    pub fn new(new_depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(new_depth),
            starting_position: Point::new(0, 0),
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
        }
    }

    fn rooms_and_corridors(&mut self, ecs: &mut World) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
            let new_room = Rect {
                x1: x,
                y1: y,
                x2: x + w,
                y2: y + h,
            };
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if other_room.intersect(&new_room) {
                    ok = false;
                }
            }

            if ok {
                apply_room_to_map(&mut self.map, &new_room);
                self.take_snapshot();

                if !self.rooms.is_empty() {
                    let new_center = new_room.center();
                    let prev_center = self.rooms[self.rooms.len() - 1].center();

                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(
                            &mut self.map,
                            prev_center.x,
                            new_center.x,
                            prev_center.y,
                        );
                        apply_vertical_tunnel(
                            &mut self.map,
                            new_center.x,
                            prev_center.y,
                            new_center.y,
                        );
                    } else {
                        apply_vertical_tunnel(
                            &mut self.map,
                            prev_center.x,
                            prev_center.y,
                            new_center.y,
                        );
                        apply_horizontal_tunnel(
                            &mut self.map,
                            prev_center.x,
                            new_center.x,
                            new_center.y,
                        );
                    }
                }
                self.rooms.push(new_room);
                self.take_snapshot();
            }
        }

        let stairs_position = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_flat(stairs_position.x, stairs_position.y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        self.starting_position = self.rooms[0].center();
    }
}
