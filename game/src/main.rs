pub mod components;
pub mod damage_system;
pub mod game;
pub mod gamelog;
pub mod gui;
pub mod inventory_system;
pub mod item_drop_system;
pub mod map;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod player;
pub mod spawner;
pub mod state;
pub mod visibility_system;

use components::AreaOfEffect;
use components::BlocksTile;
use components::CombatStats;
use components::Consumable;
use components::InBackpack;
use components::InflictsDamage;
use components::Item;
use components::Monster;
use components::Name;
use components::ProvidesHealing;
use components::Ranged;
use components::SufferDamage;
use components::WantsToDropItem;
use components::WantsToMelee;
use components::WantsToPickUpItem;
use components::WantsToUseItem;
use gamelog::GameLog;
use map::Map;

use components::Position;
use components::Renderable;
use components::Viewshed;
use game::Game;
use player::*;
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
            game.register::<Item>();
            game.register::<ProvidesHealing>();
            game.register::<InBackpack>();
            game.register::<WantsToPickUpItem>();
            game.register::<WantsToUseItem>();
            game.register::<WantsToDropItem>();
            game.register::<Consumable>();
            game.register::<Ranged>();
            game.register::<InflictsDamage>();
            game.register::<AreaOfEffect>();

            let map = Map::new_map_rooms_and_corridors();

            let player_pos = map.rooms[0].center();
            let player = spawner::spawn_player(&mut game.state.ecs, player_pos.x, player_pos.y);

            game.state.ecs.insert(rltk::RandomNumberGenerator::new());
            for room in map.rooms.iter() {
                spawner::spawn_room(&mut game.state.ecs, room);
            }

            game.state.ecs.insert(RunState::PreRun);
            game.state.ecs.insert(player);
            game.state.ecs.insert(map);
            game.state.ecs.insert(player_pos);
            game.state.ecs.insert(GameLog {
                entries: vec!["Welcome to Dangerous Deliveries".to_string()],
            });

            // move game into this function
            Game::run(game)
        }
        Err(err) => err,
    }
}
