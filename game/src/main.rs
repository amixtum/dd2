pub mod components;
pub mod damage_system;
pub mod game;
pub mod gamelog;
pub mod gui;
pub mod help_viewer;
pub mod inventory_system;
pub mod item_drop_system;
pub mod map;
pub mod map_builders;
pub mod map_indexing_system;
pub mod movement_system;
pub mod player;
pub mod spawner;
pub mod state;
pub mod visibility_system;

use components::AreaOfEffect;
use components::Balance;
use components::BlocksTile;
use components::CombatStats;
use components::Consumable;
use components::InBackpack;
use components::InflictsDamage;
use components::InstVel;
use components::Item;
use components::Monster;
use components::Name;
use components::ProvidesHealing;
use components::Ranged;
use components::SufferDamage;
use components::Velocity;
use components::WantsToDropItem;
use components::WantsToFallover;
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
use rltk::Point;
use state::RunState;

//use std::env;

const SHOW_MAPGEN_VISUALIZER: bool = true;

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
            game.register::<Velocity>();
            game.register::<InstVel>();
            game.register::<Balance>();
            game.register::<WantsToFallover>();

            let player_entity = spawner::spawn_player(&mut game.state.ecs, 0, 0);

            game.state.ecs.insert(rltk::RandomNumberGenerator::new());
            game.state.ecs.insert(RunState::MapGeneration);
            game.state.ecs.insert(player_entity);
            game.state.ecs.insert(Map::new(1));
            game.state.ecs.insert(Point::new(0, 0));
            game.state.ecs.insert(GameLog {
                entries: vec!["Welcome to Dangerous Deliveries".to_string()],
            });

            game.state.generate_world_map(1);

            // move game into this function
            Game::run(game)
        }
        Err(err) => err,
    }
}
