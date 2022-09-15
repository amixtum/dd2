use rltk::{GameState, Point, Rltk};

use specs::prelude::*;

use crate::components::{Ranged, Viewshed, WantsToDropItem, WantsToUseItem};
use crate::damage_system::DamageSystem;
use crate::gui::ItemMenuResult;
use crate::inventory_system::{ItemCollectionSystem, ItemUseSystem};
use crate::item_drop_system::ItemDropSystem;
use crate::map::Map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::{look_mode_input, ranged_targeting_input, Player};
use crate::visibility_system::VisibilitySystem;
use crate::{damage_system, gui};

use super::components::Position;
use super::components::Renderable;
use super::player::player_input;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    Looking,
    CleanupTooltips,
    ShowInventory,
    ProcessInventory,
    ShowDropItem,
    ProcessDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
        cursor: Point,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
}

pub struct State {
    pub ecs: World,
    pub map_drawn: bool,
    pub redraw_menu: bool,
    pub redraw_targeting: bool,
    pub draw_inventory: bool,
    pub look_cursor: (i32, i32),
    pub last_mouse_position: (i32, i32),
}

impl State {
    pub fn run_systems_player(&mut self) {
        let mut vis = VisibilitySystem {};
        let mut melee_system = MeleeCombatSystem {};
        let mut dmg_system = DamageSystem {};
        let mut map_index = MapIndexingSystem {};
        let mut pickup = ItemCollectionSystem {};
        let mut drop_system = ItemDropSystem {};
        let mut potion_system = ItemUseSystem {};

        potion_system.run_now(&self.ecs);
        pickup.run_now(&self.ecs);
        drop_system.run_now(&self.ecs);
        vis.run_now(&self.ecs);
        map_index.run_now(&self.ecs);
        melee_system.run_now(&self.ecs);
        dmg_system.run_now(&self.ecs);

        // update the state of the world
        self.ecs.maintain();
    }
    pub fn run_systems_monsters(&mut self) {
        let mut vis = VisibilitySystem {};
        let mut monster_ai = MonsterAI {};
        let mut map_index = MapIndexingSystem {};

        vis.run_now(&self.ecs);
        monster_ai.run_now(&self.ecs);
        map_index.run_now(&self.ecs);

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

        let last_cursor = self.look_cursor;

        match newrunstate {
            RunState::PreRun => {
                self.run_systems_player();
                newrunstate = RunState::AwaitingInput;
                self.map_drawn = false;
            },
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            },
            RunState::PlayerTurn => {
                self.run_systems_player();
                damage_system::delete_dead(&mut self.ecs);
                newrunstate = RunState::MonsterTurn;
            },
            RunState::MonsterTurn => {
                self.run_systems_monsters();
                damage_system::delete_dead(&mut self.ecs);
                newrunstate = RunState::AwaitingInput;
            },
            RunState::Looking => {
                if self.last_mouse_position.0 == -1 {
                    self.last_mouse_position = ctx.mouse_pos();
                }

                if self.look_cursor.0 == -1 {
                    let player_pos = self.ecs.fetch::<Point>();
                    self.look_cursor.0 = player_pos.x;
                    self.look_cursor.1 = player_pos.y;
                }

                let look_input = look_mode_input(self, ctx);
                newrunstate = look_input.0;
                self.look_cursor = look_input.1;
            },
            RunState::CleanupTooltips => {
                newrunstate = RunState::AwaitingInput;
                self.map_drawn = false;
            },
            RunState::ShowInventory => {
                newrunstate = RunState::ProcessInventory;
                self.draw_inventory = true;
            },
            RunState::ProcessInventory => {
                let result = gui::process_inventory(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                        self.map_drawn = false;
                    }
                    gui::ItemMenuResult::NoResponse => {
                    }
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let ranged_items = self.ecs.read_storage::<Ranged>();
                        if let Some(ranged_item) = ranged_items.get(item_entity) {
                            let player_pos = self.ecs.fetch::<Point>();
                            newrunstate = RunState::ShowTargeting {
                                range: ranged_item.range,
                                item: item_entity,
                                cursor: *player_pos,
                            };
                            self.map_drawn = false;
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent to use item");
                            newrunstate = RunState::PlayerTurn;
                            self.map_drawn = false;
                        }
                    }
                }
            },
            RunState::ShowDropItem => {
                gui::draw_drop_item_menu(self, ctx);
                newrunstate = RunState::ProcessDropItem;
            },
            RunState::ProcessDropItem => {
                let result = gui::process_drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                        self.map_drawn = false;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent to drop item");
                        newrunstate = RunState::PlayerTurn;
                        self.map_drawn = false;
                    }
                }
            },
            RunState::ShowTargeting {
                range,
                item,
                cursor,
            } => {
                let last_cursor = cursor;
                let cursor = ranged_targeting_input(self, ctx, cursor, range);
                let selection = gui::ranged_target_selection(self, ctx, cursor, range);
                match selection.0 {
                    ItemMenuResult::NoResponse => {
                        if last_cursor != cursor {
                            gui::ranged_target(self, ctx, cursor, range, item);
                        }
                        newrunstate = RunState::ShowTargeting {
                            range,
                            item,
                            cursor,
                        };
                    }
                    ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                        self.redraw_targeting = true;
                        self.map_drawn = false;
                    }
                    ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: selection.1,
                                },
                            )
                            .expect("Unable to insert intent to use ranged item");
                        newrunstate = RunState::PlayerTurn;
                        self.map_drawn = false;
                    }
                }

            },
            RunState::MainMenu { menu_selection } => {
                let result = gui::process_main_menu(self, ctx);

                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        if selected != menu_selection {
                            self.redraw_menu = true;
                        }
                        newrunstate = RunState::MainMenu { menu_selection: selected };
                    },
                    gui::MainMenuResult::Selected { selected } => {
                        if selected != menu_selection {
                            self.redraw_menu = true;
                        }
                        match selected {
                            gui::MainMenuSelection::NewGame => {
                                newrunstate = RunState::PreRun;
                            },
                            gui::MainMenuSelection::Quit => {
                                ::std::process::exit(0);
                            },
                        }
                    }
                }
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        let mut looked = false;
        if newrunstate == RunState::Looking {
            let viewsheds = self.ecs.read_storage::<Viewshed>();
            let player = self.ecs.fetch::<Entity>();
            let mouse_pos = ctx.mouse_point();

            if last_cursor.0 != self.look_cursor.0 || last_cursor.1 != self.look_cursor.1 {
                looked = true;
            } else if let Some(viewshed) = viewsheds.get(*player) {
                if (mouse_pos.x != self.last_mouse_position.0
                    || mouse_pos.y != self.last_mouse_position.1)
                    && viewshed.visible_tiles.contains(&mouse_pos)
                {
                    self.look_cursor = (mouse_pos.x, mouse_pos.y);
                    self.last_mouse_position = (mouse_pos.x, mouse_pos.y);
                    looked = true;
                }
            }
        }

        if !self.map_drawn
            || newrunstate == RunState::PlayerTurn
            || newrunstate == RunState::MonsterTurn
            || (newrunstate == RunState::Looking && looked)
        {
            self.map_drawn = true;

            // clear screen
            ctx.cls();

            Map::draw_map(&self.ecs, ctx);

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

            let viewsheds = self.ecs.read_storage::<Viewshed>();
            let players = self.ecs.read_storage::<Player>();

            for (_player, viewshed) in (&players, &viewsheds).join() {
                // draw all objects that have both a position and renderable component
                for (pos, render) in data.iter() {
                    if viewshed.visible_tiles.contains(&pos.point) {
                        ctx.set(pos.point.x, pos.point.y, render.fg, render.bg, render.glyph);
                    }
                }
            }

            gui::draw_ui(&self.ecs, ctx);

            if newrunstate == RunState::Looking && looked {
                gui::draw_tooltips_xy(&self.ecs, ctx, self.look_cursor.0, self.look_cursor.1);
            }
        }

        if self.draw_inventory {
            gui::show_inventory(self, ctx);
            self.draw_inventory = false;
        }

        match newrunstate {
            RunState::MainMenu { .. } => {
                if self.redraw_menu {
                    ctx.cls();
                    gui::draw_main_menu(self, ctx);
                    self.redraw_menu = false;
                }
            },
            RunState::ShowTargeting { range, item, cursor } => {
                if self.redraw_targeting {
                    gui::ranged_target(self, ctx, cursor, range, item);
                    self.redraw_targeting = false;
                }
            }
            _ => {},
        }
    }
}
