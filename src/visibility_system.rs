use std::borrow::BorrowMut;

use super::{ Viewshed, Position, Map, Player };

use specs::prelude::*;
use rltk::{ field_of_view, Point };


pub struct VisibilitySystem{}

impl<'a> System<'a> for VisibilitySystem {
    // ReadExpect is used here since if the map is NOT 
    // included in the SystemData, this is a failure and 
    // and is an exceptional condition.
    type SystemData = (WriteExpect<'a, Map>,
                       Entities<'a>,
                       WriteStorage<'a, Viewshed>,
                       WriteStorage<'a, Position>,
                       ReadStorage<'a, Player>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.visible_tiles.clear();
                // confusing, especially arg3
                // arg1 - the field of view's center is the entities x-y position
                // arg2 - the range is defined by the viewshed component's range 
                //        field
                // arg3 - a deferenced reference is obtained to get the map
                //        arg3 expects a reference to an object implementing the 
                //        Algorithm2D trait
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), 
                                                    viewshed.range, 
                                                    &*map);
                
                // the lambda here deletes ANY tiles that DO NOT match the criteria
                // defined in the lambda
                viewshed.visible_tiles.retain(
                    |p| p.x > 0 && p.x < map.width && p.y >= 0 && p.y < map.height
                );

                let p: Option<&Player> = player.get(ent);

                if let Some(p) = p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }

                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}