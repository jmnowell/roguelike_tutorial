use super::{Viewshed, Player};
use super::{ Rect };

use rltk::{ RGB, Rltk, RandomNumberGenerator, 
            Algorithm2D, BaseMap, Point 
        };

use specs::prelude::*;

use std::cmp::{ max, min };

// constants for map
const MAPWIDTH: usize = 80;
const MAPHEIGHT: usize = 43;
const MAPCOUNT: usize = MAPWIDTH * MAPHEIGHT;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

// trait impls
impl Algorithm2D for Map {
    fn dimensions(&self) -> rltk::Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> 
                                        rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0));
        }

        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0));
        }

        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - 1, 1.0));
        }

        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0));
        }

        // diagonals
        if self.is_exit_valid(x-1, y-1) {
            exits.push(((idx-w)-1, 1.45));
        }

        if self.is_exit_valid(x+1, y-1) {
            exits.push(((idx-w)+1, 1.45));
        }

        if self.is_exit_valid(x-1, y+1) {
            exits.push(((idx+w)-1, 1.45));
        }

        if self.is_exit_valid(x+1, y+1) {
            exits.push(((idx+w)+1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        // exit is not valid if the position here is less 
        // than 0 or greater than the edge of the max screen
        // same applies to the height.
        if x < 1 || x > self.width - 1 ||
           y < 1 || y > self.height - 1 { return false; }

        // if the position is a Wall Tile, then it is also not 
        // a valid exit.
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        // inclusive range
        // we want to visit every segment in the rooms rectangle
        // and tile it with the Floor tile
        // 
        // this needs to be done for both y and x.
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }
    
    fn apply_horizontal_tunnel(&mut self,x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2) ..= max(x1, x2) {
            let idx = self.xy_idx(x, y);
    
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..= max (y1, y2) {
            let idx = self.xy_idx(x, y);
    
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
        };
    
        const MAX_ROOMS: i32 = 30; // maximum number of rooms possible
        const MIN_SIZE: i32 = 6; // minimum room size in tiles
        const MAX_SIZE: i32 = 10; // maximum room size in tiles
    
        let mut rng = RandomNumberGenerator::new();
    
        for _ in 0..MAX_ROOMS {
            // generate a width/height for a room by obtaining
            // a value between the rang of MIN_SIZE and MAX_SIZE
            // using the Rltk random number generator
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
    
            // generate the x and y size of of the rectangle
            // in this case, we're obtaining a value between:
            //
            //  x = 1 -> the screen width minus the generated width of the 
            //           the rectangle.
            //
            //  y = 1 -> the screen height minus the generated height of 
            //           the rectangle
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
    
            // check to verify that the new room DOES NOT intersect
            // in anyway the other rooms that have previously been generated
            // due to the randomness, we are guaranteeded AT LEAST one room
            // and AT MOST the maximum rooms
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
    
            if ok {
                // bind the new room to the map
                map.apply_room_to_map(&new_room);
    
                // if there is more than one room in the vector of Rectangles.
                if !map.rooms.is_empty() {
                    // obtain the center of the new room
                    let (new_x, new_y) = new_room.center();
    
                    // obtain the center of the last added room
                    let room_len = map.rooms.len() - 1;
                    let (prev_x, prev_y) = map.rooms[room_len].center();
    
                    // randomly decide how the tunnels should be connected
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
            }
    
            map.rooms.push(new_room);
        }
    
        map
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let mut x = 0;
    let mut y = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;

            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
    
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
                }
            }
            
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }

            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }
        
        // move the coordinates a bit...
        x += 1;

        if x > MAPWIDTH as i32 - 1 {
            x = 0;
            y += 1;
        }
    }
}