use super::{Position, Player, Viewshed, TileType, State, Map, 
            RunState, CombatStats, WantsToMelee };

use rltk::{ VirtualKeyCode, Rltk, Point, console };
use specs::prelude::*;

use std::cmp::{min, max};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut pos = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    // iterate through all entities that are 
    // players, positions, and viewsheds, we are going to modify them 
    // accordingly
    for (entity, _players, pos, viewshed) in 
                (&entities, &players, &mut pos, &mut viewsheds).join() {

        // check that the position isn't beyond the map boundaries
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || 
           pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { 
            return; 
        }

        let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        // MORTAL KOMBAT!
        for potential_target in map.tile_content[dest_idx].iter() {
            let target = combat_stats.get(*potential_target);

            if let Some(_target) = target {
                wants_to_melee.insert(entity, 
                                      WantsToMelee { 
                                        target: *potential_target 
                                      })
                                      .expect("Add target failed");
            }
        }

        // collision detection
        // if the player would move to a wall, we allow
        // them to move as CLOSE to the wall as we can.
        if !map.blocked[dest_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;

            // save the currnt player position to 
            // ecs write storage as a Point
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // only move the player by one tile depending on which
    // key has been detected as pressed.
    match ctx.key {
        None => { return RunState::AwaitingInput },
        Some(key) => match key {
            // Matches here are for the N, E, S, W (cardinal directions)
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H  => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode:: L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J  => try_move_player(0, 1, &mut gs.ecs),

            // Matches here are for the diagonl movements
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            _ => { return RunState::AwaitingInput },
        },
    }

    RunState::PlayerTurn
}