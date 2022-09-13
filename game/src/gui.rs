use rltk::{RGB, Point};
use specs::prelude::*;

use crate::{components::{CombatStats, Name, Position, Viewshed}, player::Player, gamelog::GameLog, map::Map};

pub fn draw_ui(ecs: &World, ctx: &mut rltk::Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    let log = ecs.fetch::<GameLog>();

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        else {
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
                if position.point.x == mouse_pos.x && position.point.y == mouse_pos.y && viewshed.visible_tiles.contains(&mouse_pos) {
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
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), " ".to_string());
                }
                y += 1;
            }

            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "->".to_string());

            return (mouse_pos.x, mouse_pos.y)
        } else {
            let arrow_pos = Point::new(mouse_pos.x + 1, mouse_pos.y);
            let left_x = mouse_pos.x + 3;
            let mut y = mouse_pos.y;

            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), " ".to_string());
                }
                y += 1;
            }

            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "->".to_string());

            return (mouse_pos.x, mouse_pos.y)
        }
    }

    return (mouse_pos.x, mouse_pos.y)
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
                if viewshed.visible_tiles.contains(&Point::new(xc, yc)) && position.point.x == xc && position.point.y == yc {
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
                ctx.print_color(left_x, y, RGB::named(rltk::BLACK), RGB::named(rltk::GREY), s);
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(rltk::BLACK), RGB::named(rltk::GREY), " ".to_string());
                }
                y += 1;
            }

            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::BLACK), RGB::named(rltk::GREY), "->".to_string());
        } else {
            let arrow_pos = Point::new(xc + 1, yc);
            let left_x = xc + 3;
            let mut y = yc;

            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::BLACK), RGB::named(rltk::GREY), s);
                let padding = (width - s.chars().count() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::BLACK), RGB::named(rltk::GREY), " ".to_string());
                }
                y += 1;
            }

            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::BLACK), RGB::named(rltk::GREY), "->".to_string());
        }
    }
}