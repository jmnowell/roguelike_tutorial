use super::{ Map, Position, BlocksTile };
use specs::prelude::*;

pub struct MapIndexingSystem{}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, BlocksTile>,
                       Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();

        for (entity, position) in (&entities,&position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // if the entites are blocking, then update the blocking list
            let _p: Option<&BlocksTile> = blockers.get(entity);

            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // push the entity to the appropriate index slot.
            // This is a copy of the entity, so there is no need to Clone
            map.tile_content[idx].push(entity);
        }
    }
}