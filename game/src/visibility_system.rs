use crate::{components::Position, components::Viewshed, map::Map, player::Player};
use rltk::field_of_view;
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            viewshed.visible_tiles.clear();
            for tile in field_of_view(pos.point, viewshed.range, &*map) {
                viewshed.visible_tiles.insert(tile);
            }
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            if let Some(_p) = player.get(ent) {
                for vis in viewshed.visible_tiles.iter() {
                    map.revealed_tiles.insert(*vis);
                }
            }
        }
    }
}
