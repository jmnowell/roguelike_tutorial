use super::{Viewshed, Player};
use super::{ Rect };

use rltk::{ RGB, Rltk, RandomNumberGenerator, 
            Algorithm2D, BaseMap, Point 
        };

use specs::prelude::*;

use std::cmp::{ max, min };

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
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * 80) + x as usize
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
    
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..= max (y1, y2) {
            let idx = self.xy_idx(x, y);
    
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80*50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80*50],
            visible_tiles: vec![false; 80*50],
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
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
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

        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}