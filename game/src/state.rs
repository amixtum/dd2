use rltk::{GameState, Point, Rltk};

use specs::prelude::*;

use crate::components::{Ranged, Viewshed, WantsToDropItem, WantsToUseItem, InBackpack, CombatStats};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::gui::{self};
use crate::gui::ItemMenuResult;
use crate::{help_viewer, spawner};
use crate::inventory_system::{ItemCollectionSystem, ItemUseSystem};
use crate::item_drop_system::ItemDropSystem;
use crate::map::{self, Map, MAPHEIGHT, MAPWIDTH};
use crate::map_indexing_system::MapIndexingSystem;
use crate::movement_system::{FalloverSystem, MovementSystem, VelocityBalanceSystem};
use crate::player::{look_mode_input, ranged_targeting_input, Player};
use crate::visibility_system::VisibilitySystem;

use super::components::Position;
use super::components::Renderable;
use super::player::player_input;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
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
    ShowHelpMenu{
        shown: bool,
    },
    NextLevel,
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
        let mut map_index = MapIndexingSystem {};
        let mut pickup = ItemCollectionSystem {};
        let mut drop_system = ItemDropSystem {};
        let mut item_use_system = ItemUseSystem {};
        let mut speed_balance = VelocityBalanceSystem {};
        let mut move_system = MovementSystem {};
        let mut fallover_system = FalloverSystem {};
        let mut damage_system = DamageSystem {};

        item_use_system.run_now(&self.ecs);

        damage_system.run_now(&self.ecs);

        pickup.run_now(&self.ecs);
        drop_system.run_now(&self.ecs);

        speed_balance.run_now(&self.ecs);
        fallover_system.run_now(&self.ecs);

        move_system.run_now(&self.ecs);
        fallover_system.run_now(&self.ecs);

        vis.run_now(&self.ecs);

        map_index.run_now(&self.ecs);

        // update the state of the world
        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete : Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete { 
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // delete all non-persistent entities
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs.delete_entity(target).expect("Unable to delete entity");
        }

        // generate a new map with depth + 1
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            let current_depth = worldmap_resource.depth;
            *worldmap_resource = Map::new_map_rooms_and_corridors(current_depth + 1);
            worldmap = worldmap_resource.clone();
        }

        // spawn entities in rooms
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room);
        }

        // store the new player position
        let player_pos = worldmap.rooms[0].center();
        let mut ppos_res = self.ecs.write_resource::<Point>();
        *ppos_res = player_pos;

        // update the player's Position component
        let mut position_comps = self.ecs.write_storage::<Position>();
        let player_ent = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_comps.get_mut(*player_ent);
        if let Some(pos_comp) = player_pos_comp {
            pos_comp.point = player_pos;
        }

        // Notify the player and give them some health
        let mut log = self.ecs.fetch_mut::<GameLog>();
        log.entries.push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_ent);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    fn tick_help_screen(&mut self, ctx: &mut rltk::Rltk, shown: bool) -> RunState {
        // only draw screen once
        if !shown {
            help_viewer::help_screen(ctx, MAPWIDTH as u32, MAPHEIGHT as u32);
            return RunState::ShowHelpMenu { shown: true };
        }
        match ctx.key {
            None => {
                return RunState::ShowHelpMenu { shown: true };
            }
            Some(key) => match key {
                rltk::VirtualKeyCode::Escape => {
                    self.map_drawn = false;
                    return RunState::AwaitingInput;
                }
                _ => {
                    return RunState::ShowHelpMenu { shown: true };
                }
            }
        }
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
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems_player();
                map::cleanup_dead(&mut self.ecs);
                newrunstate = RunState::AwaitingInput;
                self.map_drawn = false;
            }
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }
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
            }
            RunState::CleanupTooltips => {
                newrunstate = RunState::AwaitingInput;
                self.map_drawn = false;
            }
            RunState::ShowInventory => {
                newrunstate = RunState::ProcessInventory;
                self.draw_inventory = true;
            }
            RunState::ProcessInventory => {
                let result = gui::process_inventory(self, ctx);

                match result.0 {
                    gui::ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                        self.map_drawn = false;
                    }
                    gui::ItemMenuResult::NoResponse => {}
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
            }
            RunState::ShowDropItem => {
                gui::draw_drop_item_menu(self, ctx);
                newrunstate = RunState::ProcessDropItem;
            }
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
            }
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
                        {
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
                        }

                        newrunstate = RunState::PlayerTurn;
                        self.map_drawn = false;
                    }
                }
            }
            RunState::MainMenu { menu_selection } => {
                let result = gui::process_main_menu(self, ctx);

                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        if selected != menu_selection {
                            self.redraw_menu = true;
                        }
                        newrunstate = RunState::MainMenu {
                            menu_selection: selected,
                        };
                    }
                    gui::MainMenuResult::Selected { selected } => {
                        if selected != menu_selection {
                            self.redraw_menu = true;
                        }
                        match selected {
                            gui::MainMenuSelection::NewGame => {
                                newrunstate = RunState::PreRun;
                            }
                            gui::MainMenuSelection::Quit => {
                                ::std::process::exit(0);
                            }
                        }
                    }
                }
            }
            RunState::ShowHelpMenu { shown }=> {
                newrunstate = self.tick_help_screen(ctx, shown);
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        let mut moved_look_cursor = false;
        if newrunstate == RunState::Looking {
            let viewsheds = self.ecs.read_storage::<Viewshed>();
            let player = self.ecs.fetch::<Entity>();
            let mouse_pos = ctx.mouse_point();

            if last_cursor.0 != self.look_cursor.0 || last_cursor.1 != self.look_cursor.1 {
                moved_look_cursor = true;
            } else if let Some(viewshed) = viewsheds.get(*player) {
                if (mouse_pos.x != self.last_mouse_position.0
                    || mouse_pos.y != self.last_mouse_position.1)
                    && viewshed.visible_tiles.contains(&mouse_pos)
                {
                    self.look_cursor = (mouse_pos.x, mouse_pos.y);
                    self.last_mouse_position = (mouse_pos.x, mouse_pos.y);
                    moved_look_cursor = true;
                }
            }
        }

        if !self.map_drawn
            || newrunstate == RunState::PlayerTurn
            || (newrunstate == RunState::Looking && moved_look_cursor)
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

            if newrunstate == RunState::Looking && moved_look_cursor {
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
            }
            RunState::ShowTargeting {
                range,
                item,
                cursor,
            } => {
                if self.redraw_targeting {
                    gui::ranged_target(self, ctx, cursor, range, item);
                    self.redraw_targeting = false;
                }
            }
            _ => {}
        }
    }
}
