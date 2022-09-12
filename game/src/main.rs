pub mod player;
pub mod state;
pub mod game;
pub mod map;
pub mod visibility_system;
pub mod components;
pub mod monster_ai_system;

use components::Monster;
use map::Map;

use components::Position;
use components::Renderable;
use player::*;
use game::Game;
use components::Viewshed;

//use std::env;

fn main() -> rltk::BError {
    //env::set_var("RUST_BACKTRACE", "1");

    match Game::new() {
        Ok(mut game) => {
            game.register::<Position>();
            game.register::<Renderable>();
            game.register::<Player>();
            game.register::<Viewshed>();
            game.register::<Monster>();

            let map = Map::new_map_rooms_and_corridors();

            let player_pos = map.rooms[0].center();
            game.spawn_player(player_pos.x, player_pos.y);
            game.spawn_monsters(&map);

            game.state.ecs.insert(map);

            // move game into this function
            Game::run(game)
        },
        Err(err) => err,
    }

}
