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
mod map_indexing_system;
pub use map_indexing_system::*;
mod melee_combat_system;
pub use melee_combat_system::*;
mod damage_system;
pub use damage_system::*;
mod gui;
mod gamelog;
pub use gamelog::*;

use rltk::{ GameState, Rltk, RGB, Point };
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        // run all the systems as necessary.
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        let mut map_index = MapIndexingSystem{};
        map_index.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);

        let mut dmg = DamageSystem{};
        dmg.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // clear the screen - this is what the context is
        ctx.cls();

        let mut new_run_state;

        {
            let run_state = self.ecs.fetch::<RunState>();
            new_run_state = *run_state;
        }

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }

            RunState::AwaitingInput => {
                new_run_state = player_input(self, ctx);
            }

            RunState::PlayerTurn => {
                self.run_systems();
                new_run_state = RunState::MonsterTurn;
            }

            RunState::MonsterTurn => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        damage_system::delete_the_dead(&mut self.ecs);
        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
                    .with_title("Roguelike Tutorial")
                    .build()?;

    let mut gs = State { ecs: World::new() };

    // Component Registrations
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

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
                .with(BlocksTile{})
                .with(CombatStats {
                    max_hp: 16,
                    hp: 16,
                    defense: 2,
                    power: 5,
                })
                .build();
    }

    let player_entity = gs.ecs
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
        .with(CombatStats{
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5
        })
        .build();

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_x));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog{
        entries: vec!["Welcome to Rusty Roguelike.".to_string()]
    });

    rltk::main_loop(context, gs)
}