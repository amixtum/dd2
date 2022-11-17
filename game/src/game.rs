use rltk::{BError, BTerm};

use specs::prelude::*;

use crate::{
    gui::MainMenuSelection,
    map::{MAPHEIGHT, MAPWIDTH},
    state::{RunState, State},
};

pub struct Game {
    pub context: BTerm,
    pub state: State,
}

impl Game {
    pub fn new() -> Result<Self, BError> {
        use rltk::RltkBuilder;

        let try_context = RltkBuilder::simple80x50()
            .with_dimensions(MAPWIDTH * 2, MAPHEIGHT * 2)
            .with_title("Dangerous Deliveries")
            .build();

        match try_context {
            Ok(context) => {
                //context.with_post_scanlines(true);

                let state = State {
                    ecs: World::new(),
                    map_drawn: false,
                    redraw_menu: true,
                    redraw_targeting: true,
                    draw_inventory: false,
                    look_cursor: (-1, -1),
                    last_mouse_position: (-1, -1),
                    mapgen_next_state: Some(RunState::MainMenu {
                        menu_selection: MainMenuSelection::NewGame,
                    }),
                    mapgen_history: Vec::new(),
                    mapgen_index: 0,
                    mapgen_timer: 0.0,
                };

                Ok(Game { context, state })
            }
            Err(err) => Err(Err(err)),
        }
    }

    pub fn run(game: Self) -> BError {
        rltk::main_loop(game.context, game.state)
    }
}

impl Game {
    pub fn register<T>(&mut self)
    where
        T: Component,
        T::Storage: Default,
    {
        self.state.ecs.register::<T>();
    }
}
