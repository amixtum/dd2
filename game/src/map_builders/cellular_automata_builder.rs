use rltk::{RandomNumberGenerator, Point, console, Algorithm2D};
use specs::prelude::*;
use util::vec_ops;

use crate::{map::{Map, MAPHEIGHT, MAPWIDTH, TileType, MAPCOUNT}, SHOW_MAPGEN_VISUALIZER};

use super::MapBuilder;

const MIN_ROOM_SIZE : i32 = 8;
const MIN_CONNECTED_REGION_SIZE: usize = MAPCOUNT / 3;

pub struct CellularAutomataBuilder {
    map : Map,
    starting_position : Point,
    depth: i32,
    history: Vec<Map>
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Point {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self, ecs : &mut World)  {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();

        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let idx = self.map.xy_flat(x, y);
                let roll = rng.roll_dice(1, 100);
                if roll < 55 {
                    self.map.tiles[idx] = TileType::Floor;
                }
                else {
                    self.map.tiles[idx] = TileType::Wall;
                }
            }
        }
        self.take_snapshot();

        let mut done = false;
        let mut i = 0;
        self.starting_position = Point::new(self.map.width / 2, self.map.height / 2);
        let mut start_idx = self.map.xy_flat(self.starting_position.x, self.starting_position.y);
        while !done {
            let mut newtiles = self.map.tiles.clone();
            if i % 3 == 0 {
                for y in 1..self.map.height-1 {
                    for x in 1..self.map.width-1 {
                        let idx = self.map.xy_flat(x, y);
                        if self.map.tiles[idx] == TileType::Floor {
                            newtiles[idx] = TileType::Wall;
                        }
                        else {
                            let nbrs = vec_ops::neighbors(Point::new(x, y), Point::new(0, 0), Point::new(self.map.width-1, self.map.height-1));
                            let walls_count = nbrs.iter().filter(|p| {
                                let idx = self.map.xy_flat(p.x, p.y);
                                self.map.tiles[idx] == TileType::Wall
                            }).count();

                            if walls_count > 3 {
                                newtiles[idx] = TileType::Floor;
                            }
                        }
                    }
                }
            }
            else {
                for y in 1..self.map.height-1 {
                    for x in 1..self.map.width-1 {
                        let idx = self.map.xy_flat(x, y);
                        let nbrs = vec_ops::neighbors(Point::new(x, y), Point::new(0, 0), Point::new(self.map.width-1, self.map.height-1));
                        let walls_count = nbrs.iter().filter(|p| {
                            let idx = self.map.xy_flat(p.x, p.y);
                            self.map.tiles[idx] == TileType::Wall
                        }).count();

                        if walls_count == 0 || walls_count > 4 {
                            newtiles[idx] = TileType::Wall;
                        }
                        else {
                            newtiles[idx] = TileType::Floor;
                        }
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.take_snapshot();

            self.map.blocked_tiles.clear();
            self.map.populate_blocked();

            if i % 3 != 0 {
                self.starting_position = Point::new(self.map.width / 2, self.map.height / 2);
                start_idx = self.map.xy_flat(self.starting_position.x, self.starting_position.y);
                while self.map.tiles[start_idx] != TileType::Floor {
                    self.starting_position.x -= 1;
                    start_idx = self.map.xy_flat(self.starting_position.x, self.starting_position.y);
                }

                let map_starts = vec![start_idx];
                let dijkstra_map = rltk::DijkstraMap::new(self.map.width, self.map.height, &map_starts, &self.map, 200.0);
                let reachable_count = dijkstra_map.map.iter().filter(|d| {
                    **d != std::f32::MAX
                }).count();

                console::log(format!("{} reacheable", reachable_count));

                if reachable_count >= MIN_CONNECTED_REGION_SIZE {
                    done = true;
                    console::log("Finished building");
                }
            }


            i += 1;
        }

        self.map.blocked_tiles.clear();
        self.map.populate_blocked();

        let map_starts = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(self.map.width, self.map.height, &map_starts, &self.map, 200.0);
        let mut exit_tile = (0, 0.0f32);

        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let dist_to_start = dijkstra_map.map[i];

                if dist_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                }
                else {
                    // find the furthest reacheable tile and make that the exit
                    if dist_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = dist_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.take_snapshot();
    }

    fn spawn_entities(&mut self, ecs : &mut World) {
        // We need to rewrite this, too.
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

impl CellularAutomataBuilder {
    pub fn new(new_depth : i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder{
            map : Map::new(new_depth),
            starting_position : Point::new(0, 0),
            depth : new_depth,
            history: Vec::new(),
        }
    }
}