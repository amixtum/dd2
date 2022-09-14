use rltk::{BTerm, BError};

use specs::prelude::*;

use crate::state::State;


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
            Ok(mut context) => {
                //context.with_post_scanlines(true);

                let state = State {
                    ecs: World::new(),
                    has_drawn: false,
                    look_cursor: (-1, -1),
                    last_mouse_position: (-1, -1),
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


}