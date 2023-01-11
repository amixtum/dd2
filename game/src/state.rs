use rltk::{GameState, Point, Rltk};

use specs::prelude::*;

use crate::components::{
    CombatStats, InBackpack, Ranged, Viewshed, WantsToDropItem, WantsToUseItem,
};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::gui::{self};
use crate::gui::{ItemMenuResult, MainMenuSelection};
use crate::inventory_system::{ItemCollectionSystem, ItemUseSystem};
use crate::item_drop_system::ItemDropSystem;
use crate::map::{self, Map, MAPHEIGHT, MAPWIDTH};
use crate::map_indexing_system::MapIndexingSystem;
use crate::movement_system::{FalloverSystem, MovementSystem, VelocityBalanceSystem};
use crate::player::{look_mode_input, ranged_targeting_input, Player};
use crate::visibility_system::VisibilitySystem;
use crate::{help_viewer, map_builders, SHOW_MAPGEN_VISUALIZER};

use super::components::Position;
use super::components::Renderable;
use super::player::player_input;

const REVEAL_MAP: bool = true;

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
    ShowHelpMenu {
        shown: bool,
    },
    NextLevel,
    MapGeneration,
}

pub struct State {
    pub ecs: World,

    // sentinels to make sure we don't draw more than we need to
    pub map_drawn: bool,
    pub redraw_menu: bool,
    pub redraw_targeting: bool,
    pub draw_inventory: bool,

    // lookmode variables
    pub look_cursor: (i32, i32),
    pub last_mouse_position: (i32, i32),

    // mapgen variables
    pub mapgen_next_state: Option<RunState>, // where to go after mapgen
    pub mapgen_history: Vec<Map>,            // copy of the mapgen history
    pub mapgen_index: usize,                 // current index into mapgen history
    pub mapgen_timer: f32,                   // times mapgen animation
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

        let mut to_delete: Vec<Entity> = Vec::new();
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

    fn goto_next_level(&mut self, new_depth: i32) {
        // delete all non-persistent entities
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        // generate new map
        self.generate_world_map(new_depth);

        // Notify the player and give them some health
        let player_ent = self.ecs.fetch::<Entity>();
        let mut log = self.ecs.fetch_mut::<GameLog>();
        log.entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_ent);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    pub fn generate_world_map(&mut self, new_depth: i32) {
        // reset mapgen vars
        self.mapgen_history.clear();
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;

        let mut builder = map_builders::random_builder(new_depth);
        builder.build_map(&mut self.ecs);

        // clone mapgen history from new map
        self.mapgen_history = builder.get_snapshot_history();

        let ecs_ptr = &mut self.ecs as *mut World;

        let player_start;
        unsafe {
            let mut worldmap_resource = (*ecs_ptr).write_resource::<Map>();
            let mut player_position = (*ecs_ptr).write_resource::<Point>();
            *worldmap_resource = builder.get_map();
            *player_position = builder.get_starting_position();
            player_start = *player_position;
        }

        builder.spawn_entities(&mut self.ecs);

        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(pos_comp) = player_pos_comp {
            pos_comp.point = Point::new(player_start.x, player_start.y);
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
            },
        }
    }

    fn tick_prerun(&mut self) -> RunState {
        self.run_systems_player();
        self.map_drawn = false;

        RunState::AwaitingInput
    }

    fn tick_player_turn(&mut self) -> RunState {
        self.run_systems_player();
        map::cleanup_dead(&mut self.ecs);
        self.map_drawn = false;
        RunState::AwaitingInput
    }

    fn tick_next_level(&mut self) -> RunState {
        let current_depth = self.ecs.read_resource::<Map>().depth;
        self.goto_next_level(current_depth + 1);
        RunState::PreRun
    }

    fn tick_looking(&mut self, ctx: &mut Rltk) -> RunState {
        if self.last_mouse_position.0 == -1 {
            self.last_mouse_position = ctx.mouse_pos();
        }

        if self.look_cursor.0 == -1 {
            let player_pos = self.ecs.fetch::<Point>();
            self.look_cursor.0 = player_pos.x;
            self.look_cursor.1 = player_pos.y;
        }

        let look_input = look_mode_input(self, ctx);

        self.look_cursor = look_input.1;
        look_input.0
    }

    fn tick_process_inventory(&mut self, ctx: &mut Rltk) -> RunState {
        let result = gui::process_inventory(self, ctx);

        match result.0 {
            gui::ItemMenuResult::Cancel => {
                self.map_drawn = false;
                return RunState::AwaitingInput;
            }
            gui::ItemMenuResult::NoResponse => {
                return RunState::ProcessInventory;
            }
            gui::ItemMenuResult::Selected => {
                let item_entity = result.1.unwrap();
                let ranged_items = self.ecs.read_storage::<Ranged>();
                if let Some(ranged_item) = ranged_items.get(item_entity) {
                    let player_pos = self.ecs.fetch::<Point>();
                    self.map_drawn = false;
                    return RunState::ShowTargeting {
                        range: ranged_item.range,
                        item: item_entity,
                        cursor: *player_pos,
                    };
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
                    self.map_drawn = false;

                    return RunState::PlayerTurn;
                }
            }
        }
    }

    fn tick_process_drop_item(&mut self, ctx: &mut Rltk) -> RunState {
        let result = gui::process_drop_item_menu(self, ctx);
        match result.0 {
            gui::ItemMenuResult::Cancel => {
                self.map_drawn = false;
                return RunState::AwaitingInput;
            }
            gui::ItemMenuResult::NoResponse => {
                return RunState::ProcessDropItem;
            }
            gui::ItemMenuResult::Selected => {
                let item_entity = result.1.unwrap();
                let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                intent
                    .insert(
                        *self.ecs.fetch::<Entity>(),
                        WantsToDropItem { item: item_entity },
                    )
                    .expect("Unable to insert intent to drop item");

                self.map_drawn = false;
                return RunState::PlayerTurn;
            }
        }
    }

    fn tick_show_targeting(
        &mut self,
        ctx: &mut Rltk,
        range: i32,
        item: Entity,
        cursor: Point,
    ) -> RunState {
        let last_cursor = cursor;
        let cursor = ranged_targeting_input(self, ctx, cursor, range);
        let selection = gui::ranged_target_selection(self, ctx, cursor, range);
        match selection.0 {
            ItemMenuResult::NoResponse => {
                if last_cursor != cursor {
                    gui::ranged_target(self, ctx, cursor, range, item);
                }
                return RunState::ShowTargeting {
                    range,
                    item,
                    cursor,
                };
            }
            ItemMenuResult::Cancel => {
                self.redraw_targeting = true;
                self.map_drawn = false;
                return RunState::AwaitingInput;
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

                self.map_drawn = false;
                return RunState::PlayerTurn;
            }
        }
    }

    fn tick_main_menu(&mut self, ctx: &mut Rltk, menu_selection: MainMenuSelection) -> RunState {
        let result = gui::process_main_menu(self, ctx);

        match result {
            gui::MainMenuResult::NoSelection { selected } => {
                if selected != menu_selection {
                    self.redraw_menu = true;
                }
                return RunState::MainMenu {
                    menu_selection: selected,
                };
            }
            gui::MainMenuResult::Selected { selected } => {
                if selected != menu_selection {
                    self.redraw_menu = true;
                }
                match selected {
                    gui::MainMenuSelection::NewGame => {
                        return RunState::PreRun;
                    }
                    gui::MainMenuSelection::Quit => {
                        ::std::process::exit(0);
                    }
                }
            }
        }
    }

    fn tick_map_generation(&mut self, ctx: &mut Rltk) -> RunState {
        if !SHOW_MAPGEN_VISUALIZER {
            return self.mapgen_next_state.unwrap();
        }
        ctx.cls();

        Map::draw_map(&self.mapgen_history[self.mapgen_index], &self.ecs, ctx);
        self.mapgen_timer += ctx.frame_time_ms;
        if self.mapgen_timer >= 150.0 {
            self.mapgen_timer = 0.0;
            self.mapgen_index += 1;
            if self.mapgen_index >= self.mapgen_history.len() {
                return self.mapgen_next_state.unwrap();
            }
        }
        RunState::MapGeneration
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
                if REVEAL_MAP {
                    let mut map = self.ecs.fetch_mut::<Map>();
                    for y in 0..map.height {
                        for x in 0..map.width {
                            map.revealed_tiles.insert(Point::new(x, y));
                        }
                    }
                }
                newrunstate = self.tick_prerun();
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                newrunstate = self.tick_player_turn();
            }
            RunState::NextLevel => {
                newrunstate = self.tick_next_level();
            }
            RunState::Looking => {
                newrunstate = self.tick_looking(ctx);
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
                newrunstate = self.tick_process_inventory(ctx);
            }
            RunState::ShowDropItem => {
                gui::draw_drop_item_menu(self, ctx);
                newrunstate = RunState::ProcessDropItem;
            }
            RunState::ProcessDropItem => {
                newrunstate = self.tick_process_drop_item(ctx);
            }
            RunState::ShowTargeting {
                range,
                item,
                cursor,
            } => {
                newrunstate = self.tick_show_targeting(ctx, range, item, cursor);
            }
            RunState::MainMenu { menu_selection } => {
                newrunstate = self.tick_main_menu(ctx, menu_selection);
            }
            RunState::ShowHelpMenu { shown } => {
                newrunstate = self.tick_help_screen(ctx, shown);
            }
            RunState::MapGeneration => {
                newrunstate = self.tick_map_generation(ctx);
            }
        } // done determining newrunstate

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

            Map::draw_map(&self.ecs.fetch::<Map>(), &self.ecs, ctx);

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
