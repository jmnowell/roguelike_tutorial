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
            // if the entities viewshed is dirty, meaning the entity has moved
            // we run the process to determine what new tiles are visible to the 
            // entity
            if viewshed.dirty {
                viewshed.visible_tiles.clear();
                // confusing, especially arg3
                // arg1 - the field of view's center is the entities x-y 
                //        position
                //
                // arg2 - the range is defined by the viewshed component's 
                //        range field
                //
                // arg3 - a deferenced reference is obtained to get the map
                //        arg3 expects a reference to an object implementing 
                //        the Algorithm2D trait
                //
                //  arg3 is specified with a referenced to a deferenced struct
                //  because of the dyn keyword.  It needs a reference to the 
                //  actual memory location since it is assuming that the map
                //  implements the Algorithm2D trait, but it doesn't care 
                //  about the underlying datatype.
                //
                //  the map also implements the BaseMap is_opaque function
                //  which returns true if the tile type is a Wall
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), 
                                                    viewshed.range, 
                                                    &*map);
                
                // the lambda to the retain() method means that we are keeping
                // ONLY the tiles the fit the lambda's criteria.
                //
                // in this case, if the point's x > and less than the maps 
                // width and the same for the maps height.
                viewshed.visible_tiles.retain(
                    |p| p.x > 0 && p.x < map.width && 
                        p.y >= 0 && p.y < map.height
                );

                // this is only for the Player entity.
                // 
                // if the entity is a player, than we want to 
                // set some options that other entities won't 
                // care about.
                let p: Option<&Player> = player.get(ent);

                if let Some(p) = p {
                    // set all visible tiles in the map to false
                    for t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }

                    // iterate through the viewshed and if 
                    // obtain the index of the visible tiles
                    // from the viewshed and set the tiles to 
                    // true for both revealed and visible, since
                    // the visible tiles in the viewshed are within the 
                    // player's view range.
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