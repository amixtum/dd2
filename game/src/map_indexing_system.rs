use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.blocked_tiles.clear();
        map.clear_content_index();

        map.populate_blocked();

        for (entity, position) in (&entities, &position).join() {
            if let Some(_p) = blockers.get(entity) {
                map.blocked_tiles.insert(position.point);
            }

            let idx = map.xy_flat(position.point.x, position.point.y);
            map.tile_content[idx].push(entity);
        }
    }
}
