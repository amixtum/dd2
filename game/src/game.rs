use std::collections::HashSet;

use rltk::{Point, RGB, BTerm, BError, FontCharType, RandomNumberGenerator};

use specs::prelude::*;

use crate::components::Monster;
use crate::map::Map;
use crate::state::State;
use crate::components::Position;
use crate::components::Renderable;
use crate::player::Player;
use crate::components::Viewshed;

pub struct Game {
    pub context: BTerm,
    pub state: State,
}

impl Game {
    pub fn new() -> Result<Self, BError> {
        use rltk::RltkBuilder;

        let try_context = RltkBuilder::simple80x50()
            .with_title("Roguelike Tutorial")
            .build();
        
        match try_context {
            Ok(context) => {
                let state = State {
                    ecs: World::new(),
                    needs_redraw: false,
                    has_drawn: false,
                };

                Ok(Game {
                    context,
                    state,
                })
            },
            Err(err) => Err(Err(err)),
        }
    }

    pub fn run(game: Self) -> BError {
        rltk::main_loop(game.context, game.state)
    }
}

impl Game
{
    pub fn register<T>(&mut self) 
    where T: Component,
          T::Storage: Default,
    {
        self.state.ecs.register::<T>();
    }

    pub fn spawn_player(&mut self, x: i32, y: i32) {
        self.state.ecs.create_entity()
            .with(Position {point: Point::from_tuple((x, y))})
            .with(Renderable {
                glyph: rltk::to_cp437('☻'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Player {})
            .with(Viewshed {
                visible_tiles: HashSet::new(),
                range: 8,
            })
            .build();
    }

    pub fn spawn_monsters(&mut self, map: &Map) {
        let mut rng = RandomNumberGenerator::new();
        for room in map.rooms.iter().skip(1) {
            let center = room.center();
            let glyph: FontCharType;
            let roll = rng.roll_dice(1, 2);
            match roll  {
                1 => {glyph = rltk::to_cp437('g');},
                _ => {glyph = rltk::to_cp437('o');},
            }
            self.state.ecs.create_entity()
                .with(Position {point: center})
                .with(Renderable {
                    glyph,
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK),
                })
                .with(Viewshed {visible_tiles: HashSet::new(), range: 8})
                .with(Monster{})
                .build();
        }
    }
}