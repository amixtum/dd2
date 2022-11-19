use rltk::{Point, Rect};

use crate::map::{Map, TileType, MAPHEIGHT, MAPWIDTH};

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            let idx = map.xy_flat(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    apply_tunnel(map, x1, y, x2, y);
}

pub fn apply_vertical_tunnel(map: &mut Map, x: i32, y1: i32, y2: i32) {
    apply_tunnel(map, x, y1, x, y2);
}

pub fn apply_tunnel(map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
    for point in rltk::line2d_bresenham(Point::new(x1, y1), Point::new(x2, y2)) {
        if point.x < MAPWIDTH as i32 && point.y < MAPHEIGHT as i32 {
            let idx = map.xy_flat(point.x, point.y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

