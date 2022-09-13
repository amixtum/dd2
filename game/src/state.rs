use rltk::{GameState, Rltk};

use specs::prelude::*;

use crate::damage_system::DamageSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::{damage_system};
use crate::map::{Map};
use crate::map_indexing_system::MapIndexingSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::Player;
use crate::components::Viewshed;
use crate::visibility_system::VisibilitySystem;

use super::player::player_input;
use super::components::Position;
use super::components::Renderable;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Running,
}

pub struct State {
    pub ecs: World,
    pub has_drawn: bool,
}

impl State {
    pub fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        let mut monster_ai = MonsterAI{};
        let mut melee_system = MeleeCombatSystem{};
        let mut dmg_system = DamageSystem{};
        let mut map_index = MapIndexingSystem{};

        // run the visibility system on the World

        vis.run_now(&self.ecs);

        map_index.run_now(&self.ecs);

        monster_ai.run_now(&self.ecs);

        melee_system.run_now(&self.ecs);
        dmg_system.run_now(&self.ecs);



        // update the state of the world
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate; 
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            },
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            },
            RunState::Running => {
                self.run_systems();
                damage_system::delete_dead(&mut self.ecs);
                newrunstate = RunState::AwaitingInput;
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        if !self.has_drawn || newrunstate == RunState::Running {
            self.has_drawn = true;

            // clear screen
            ctx.cls();

            Map::draw_map(&self.ecs, ctx);

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let viewsheds = self.ecs.read_storage::<Viewshed>();
            let players = self.ecs.read_storage::<Player>();

            for (_player, viewshed) in (&players, &viewsheds).join() {
                // draw all objects that have both a position and renderable component
                for (pos, render) in (&positions, &renderables).join() {
                    if viewshed.visible_tiles.contains(&pos.point) {
                        ctx.set(pos.point.x, pos.point.y, render.fg, render.bg, render.glyph);
                    }
                }
            }
        }

    }
}