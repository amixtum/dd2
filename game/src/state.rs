use rltk::{GameState, Rltk};

use specs::prelude::*;

use crate::map::{TileType, Map};
use crate::monster_ai_system::MonsterAI;
use crate::player::Player;
use crate::components::Viewshed;
use crate::visibility_system::VisibilitySystem;

use super::player::player_input;
use super::components::Position;
use super::components::Renderable;

pub struct State {
    pub ecs: World,
    pub needs_redraw: bool,
    pub has_drawn: bool,
}

impl State {
    pub fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        let mut monster_ai = MonsterAI{};

        // run the visibility system on the World
        vis.run_now(&self.ecs);
        monster_ai.run_now(&self.ecs);

        // update the state of the world
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        self.needs_redraw = false;

        // this sets needs redraw
        player_input(self, ctx);

        if !self.has_drawn {
            self.needs_redraw = true;
        }

        if self.needs_redraw {
            // clear screen
            ctx.cls();

            self.run_systems();

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