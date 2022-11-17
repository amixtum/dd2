use rltk::RGB;

pub fn help_screen(ctx: &mut rltk::Rltk, width: u32, _height: u32) {
    ctx.cls();

    let mut left_col = Vec::new();
    let mut right_col = Vec::new();

    left_col.push((String::from("Movement"), Some(RGB::named(rltk::CYAN))));
    right_col.push(String::from(""));

    left_col.push((String::from("Up"), None));
    right_col.push(String::from("W or K"));

    left_col.push((String::from("Left"), None));
    right_col.push(String::from("A or H"));

    left_col.push((String::from("Down"), None));
    right_col.push(String::from("S or J"));

    left_col.push((String::from("Right"), None));
    right_col.push(String::from("D or L"));

    left_col.push((String::from("NorthEast"), None));
    right_col.push(String::from("E or U"));

    left_col.push((String::from("NorthWest"), None));
    right_col.push(String::from("Q or Y"));

    left_col.push((String::from("SouthWest"), None));
    right_col.push(String::from("Z or B"));

    left_col.push((String::from("SouthEast"), None));
    right_col.push(String::from("C or N"));

    left_col.push((String::from("Wait (don't increase speed or turn)"), None));
    right_col.push(String::from("1 (one) or Period"));

    left_col.push((String::from("Game Functions"), Some(RGB::named(rltk::CYAN))));
    right_col.push(String::from(""));

    left_col.push((String::from("Look Mode"), None));
    right_col.push(String::from("Semicolon"));

    left_col.push((String::from("Get Item"), None));
    right_col.push(String::from("G"));

    left_col.push((String::from("Drop Item"), None));
    right_col.push(String::from("Minus"));

    left_col.push((String::from("Inventory"), None));
    right_col.push(String::from("I"));

    left_col.push((String::from("Go Down Stairs"), None));
    right_col.push(String::from("0 (zero)"));

    //left_col.push((String::from("Get Information or Give Package"), None));
    //right_col.push(String::from("G"));

    //left_col.push(String::from("Game Functions"));
    //right_col.push(String::from(""));

    //left_col.push(String::from("New Game"));
    //right_col.push(String::from("5"));

    //left_col.push(String::from(""));
    //right_col.push(String::from("Esc"));

    //left_col.push(String::from("Exit Game"));
    //right_col.push(String::from("Ctrl+C"));

    let mut col = 0;
    let mut sc_y = 0;

    while col < left_col.len() && col < right_col.len() {
        match &left_col[col].1 {
            None => {
                ctx.print(1, sc_y, &left_col[col].0);
            }
            Some(color) => {
                ctx.print_color(1, sc_y, *color, RGB::named(rltk::BLACK), &left_col[col].0);
            }
        }

        ctx.print(width as i32 / 2, sc_y, &right_col[col]);
        sc_y += 1;
        for x in 0..width {
            ctx.set(
                x,
                sc_y,
                RGB::named(rltk::DARKGRAY),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('─'),
            );
        }
        sc_y += 1;
        col += 1;
    }

    ctx.print_color(
        1,
        sc_y,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Color Coding",
    );

    sc_y += 2;

    ctx.print_color(
        1,
        sc_y,
        RGB::from_u8(0, 255, 0),
        RGB::named(rltk::BLACK),
        "Fall Over or Game Over",
    );

    sc_y += 2;

    let title = "Balance Gradient: ";
    let ds = "Decrease Balance, ";
    let is = "Increase Balance";

    ctx.print_color(
        1,
        sc_y,
        RGB::from_u8(255, 0, 255),
        RGB::named(rltk::BLACK),
        title,
    );

    ctx.print_color(
        title.chars().count() as i32 + 1,
        sc_y,
        RGB::from_u8(0, 0, 255),
        RGB::named(rltk::BLACK),
        ds,
    );

    ctx.print_color(
        title.chars().count() as i32 + ds.chars().count() as i32 + 2,
        sc_y,
        RGB::from_u8(255, 0, 0),
        RGB::named(rltk::BLACK),
        is,
    );

    sc_y += 2;

    ctx.print_color(
        1,
        sc_y,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        "Press a movement key to increase speed or turn.",
    );

    sc_y += 2;

    /*
    ctx.print_color(
        1,
        sc_y,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        "Press G when near another skater to talk to them",
    );

    sc_y += 2;
    */

    ctx.print(1, sc_y, "Esc to Return");
}
