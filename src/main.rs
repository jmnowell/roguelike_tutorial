mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
pub use visibility_system::*;
mod monster_ai_system;
pub use monster_ai_system::*;

use rltk::{ GameState, Rltk, RGB, Point };
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl State {
    fn run_systems(&mut self) {
        // run all the systems as necessary.
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // clear the screen - this is what the context is
        ctx.cls();

        if self.runstate == RunState::Running {
            // update all the systems in the ECS
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            // capture the player input from the keyboard if any
            // and try to move the player entity per the player_input
            // and the try_move_player function
            self.runstate = player_input(self, ctx)
        }

        // redraw the map - the map is only drawn based on the 
        // visible/revealed tiles.
        draw_map(&self.ecs, ctx);

        // update the positions on the map for any component that has 
        // both Position and Renderable.  This includes the player.
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
                    .with_title("Roguelike Tutorial")
                    .build()?;

    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };

    // Component Registrations
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let mut rng = rltk::RandomNumberGenerator::new();

    // populate the dungeon with MONSTERS!
    // g - Goblin
    // o - for WE DA ORKS!
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);

        match roll {
            1 => { 
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            },
            _ => { 
                glyph = rltk::to_cp437('o');
                name = "Ork".to_string();
            },
        }

        gs.ecs.create_entity()
                .with(Position{ x, y})
                .with(Renderable{
                    glyph: glyph,
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK)
                })
                .with(Viewshed{
                    visible_tiles: Vec::new(),
                    range: 8,
                    dirty: true
                })
                .with(Monster{})
                .with(Name{ name: format!("{} #{}", &name, i)})
                .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_x));

    gs.ecs
        .create_entity()
        .with(Position{ x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed{ 
            visible_tiles: Vec::new(), 
            range: 8,
            dirty: true,
        })
        .with(Name{ name: "Player".to_string() })
        .build();

    rltk::main_loop(context, gs)
}