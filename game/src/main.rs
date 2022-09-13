pub mod components;
pub mod player;
pub mod state;
pub mod game;
pub mod map;
pub mod visibility_system;
pub mod monster_ai_system;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod damage_system;
pub mod gui;


use components::BlocksTile;
use components::CombatStats;
use components::Monster;
use components::Name;
use components::SufferDamage;
use components::WantsToMelee;
use map::Map;

use components::Position;
use components::Renderable;
use player::*;
use game::Game;
use components::Viewshed;
use state::RunState;

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
            game.register::<Name>();
            game.register::<BlocksTile>();
            game.register::<CombatStats>();
            game.register::<WantsToMelee>();
            game.register::<SufferDamage>();

            let map = Map::new_map_rooms_and_corridors();

            let player_pos = map.rooms[0].center();
            let player = game.spawn_player(player_pos.x, player_pos.y);
            game.spawn_monsters(&map);

            game.state.ecs.insert(RunState::PreRun);
            game.state.ecs.insert(player);
            game.state.ecs.insert(map);
            game.state.ecs.insert(player_pos);

            // move game into this function
            Game::run(game)
        },
        Err(err) => err,
    }

}
