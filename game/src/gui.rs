use std::collections::HashSet;

use rltk::{Point, VirtualKeyCode, RGB};
use specs::prelude::*;

use crate::{
    components::{AreaOfEffect, CombatStats, InBackpack, Name, Position, Viewshed},
    gamelog::GameLog,
    map::Map,
    player::Player,
    state::{RunState, State},
};

#[derive(PartialEq, Clone, Copy)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuSelection {
    NewGame,
    Quit,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn show_inventory(gs: &mut State, ctx: &mut rltk::Rltk) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // count the Items attached to the player's backpack and their Names
    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let item_count = inventory.count();

    let mut y = (25 - (item_count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (item_count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + item_count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut letter_code_idx = 0;
    for (_entity, _item, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + letter_code_idx as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());

        y += 1;
        letter_code_idx += 1;
    }
}

pub fn process_inventory(gs: &mut State, ctx: &mut rltk::Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // count the Items attached to the player's backpack and their Names
    let inventory = (&backpack)
        .join()
        .filter(|item| item.owner == *player_entity);
    let item_count = inventory.count();

    let mut equippable = Vec::new();
    for (entity, _item) in (&entities, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        equippable.push(entity);
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                return (ItemMenuResult::Cancel, None);
            }
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < item_count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                return (ItemMenuResult::NoResponse, None);
            }
        },
    }
}

pub fn process_drop_item_menu(
    gs: &mut State,
    ctx: &mut rltk::Rltk,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack)
        .join()
        .filter(|item| item.owner == *player_entity);
    let count = inventory.count();

    let mut equippable: Vec<Entity> = Vec::new();
    for (entity, _pack) in (&entities, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        equippable.push(entity);
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn draw_drop_item_menu(gs: &mut State, ctx: &mut rltk::Rltk) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Drop Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut j = 0;
    for (_entity, _pack, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        y += 1;
        j += 1;
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut rltk::Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );

        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    let log = ecs.fetch::<GameLog>();

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        } else {
            break;
        }
        y += 1;
    }
}

pub fn draw_tooltips_mouse(ecs: &World, ctx: &mut rltk::Rltk) -> (i32, i32) {
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let player = ecs.fetch::<Entity>();

    let mouse_pos = ctx.mouse_point();
    if mouse_pos.x >= map.width || mouse_pos.y >= map.height {
        return (-1, -1);
    }

    let mut tooltip = Vec::new();

    if let Some(viewshed) = viewsheds.get(*player) {
        for (name, position) in (&names, &positions).join() {
            if position.point.x == mouse_pos.x
                && position.point.y == mouse_pos.y
                && viewshed.visible_tiles.contains(&mouse_pos)
            {
                tooltip.push(name.name.to_string());
                break;
            }
        }
    }

    if !tooltip.is_empty() {
        let mut width = 0;
        for s in tooltip.iter() {
            let len = s.chars().count();
            if width < len as i32 {
                width = len as i32;
            }
        }
        // for the arrow
        width += 3;

        if mouse_pos.x > 40 {
            let arrow_pos = Point::new(mouse_pos.x - 2, mouse_pos.y);
            let left_x = mouse_pos.x - width;
            let mut y = mouse_pos.y;

            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        " ".to_string(),
                    );
                }
                y += 1;
            }

            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                "->".to_string(),
            );

            return (mouse_pos.x, mouse_pos.y);
        } else {
            let arrow_pos = Point::new(mouse_pos.x + 1, mouse_pos.y);
            let left_x = mouse_pos.x + 3;
            let mut y = mouse_pos.y;

            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        " ".to_string(),
                    );
                }
                y += 1;
            }

            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                "->".to_string(),
            );

            return (mouse_pos.x, mouse_pos.y);
        }
    }

    return (mouse_pos.x, mouse_pos.y);
}

pub fn draw_tooltips_xy(ecs: &World, ctx: &mut rltk::Rltk, xc: i32, yc: i32) {
    ctx.set_bg(xc, yc, RGB::named(rltk::MAGENTA));

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let player = ecs.fetch::<Entity>();

    if xc >= map.width || yc >= map.height {
        return;
    }

    let mut tooltip = Vec::new();

    if let Some(viewshed) = viewsheds.get(*player) {
        for (name, position) in (&names, &positions).join() {
            if viewshed.visible_tiles.contains(&Point::new(xc, yc))
                && position.point.x == xc
                && position.point.y == yc
            {
                tooltip.push(name.name.to_string());
                break;
            }
        }
    }

    if !tooltip.is_empty() {
        let mut width = 0;
        for s in tooltip.iter() {
            let len = s.chars().count();
            if width < len as i32 {
                width = len as i32;
            }
        }
        // for the arrow
        width += 3;

        if xc > 40 {
            let arrow_pos = Point::new(xc - 2, yc);
            let left_x = xc - width;
            let mut y = yc;

            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::BLACK),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::BLACK),
                        RGB::named(rltk::GREY),
                        " ".to_string(),
                    );
                }
                y += 1;
            }

            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::BLACK),
                RGB::named(rltk::GREY),
                "->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(xc + 1, yc);
            let left_x = xc + 3;
            let mut y = yc;

            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::BLACK),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::BLACK),
                        RGB::named(rltk::GREY),
                        " ".to_string(),
                    );
                }
                y += 1;
            }

            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::BLACK),
                RGB::named(rltk::GREY),
                "->".to_string(),
            );
        }
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut rltk::Rltk,
    cursor: Point,
    range: i32,
    item: Entity,
) {
    let map = gs.ecs.fetch::<Map>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Press Enter to select:",
    );

    let mut available_cells = HashSet::new();
    if let Some(visible) = viewsheds.get(*player_entity) {
        for pos in visible.visible_tiles.iter() {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(*pos, *player_pos);
            if dist <= range as f32 {
                ctx.set_bg(pos.x, pos.y, RGB::named(rltk::BLUE));
                available_cells.insert(*pos);
            }
        }
    }

    if let Some(aoe) = gs.ecs.read_storage::<AreaOfEffect>().get(item) {
        let aoe_tiles = rltk::field_of_view(cursor, aoe.radius, &*map);
        for tile in aoe_tiles.iter() {
            ctx.set_bg(tile.x, tile.y, RGB::named(rltk::ORANGE));
        }
    }

    /*

    */

    let valid_target = available_cells.contains(&cursor);

    if valid_target {
        ctx.set_bg(cursor.x, cursor.y, RGB::named(rltk::CYAN));
    } else {
        ctx.set_bg(cursor.x, cursor.y, RGB::named(rltk::GREY));
    }
}

pub fn ranged_target_selection(
    gs: &mut State,
    ctx: &mut rltk::Rltk,
    cursor: Point,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    let mut available_cells = HashSet::new();
    if let Some(visible) = viewsheds.get(*player_entity) {
        for pos in visible.visible_tiles.iter() {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(*pos, *player_pos);
            if dist <= range as f32 {
                available_cells.insert(*pos);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    let valid_target = available_cells.contains(&cursor);

    if valid_target {
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Return => {
                    return (ItemMenuResult::Selected, Some(cursor));
                }
                VirtualKeyCode::Escape => {
                    return (ItemMenuResult::Cancel, None);
                }
                _ => {}
            },
        }
    } else {
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::Return => {
                    return (ItemMenuResult::Cancel, None);
                }
                _ => {}
            },
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn draw_main_menu(gs: &State, ctx: &mut rltk::Rltk) {
    let runstate = gs.ecs.fetch::<RunState>();

    if let RunState::MainMenu { menu_selection } = *runstate {
        ctx.print_color_centered(
            15,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            "Dangerous Deliveries",
        );
        if menu_selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(
                24,
                RGB::named(rltk::MAGENTA),
                RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        } else {
            ctx.print_color_centered(
                24,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        }

        if menu_selection == MainMenuSelection::Quit {
            ctx.print_color_centered(
                26,
                RGB::named(rltk::MAGENTA),
                RGB::named(rltk::BLACK),
                "Quit",
            );
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }
    }
}

pub fn process_main_menu(gs: &mut State, ctx: &mut rltk::Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    if let RunState::MainMenu { menu_selection } = *runstate {
        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: menu_selection,
                }
            }
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    let newselection;
                    match menu_selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Down => {
                    let newselection;
                    match menu_selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: menu_selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: menu_selection,
                    }
                }
            },
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}
