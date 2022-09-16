use std::collections::HashSet;
use std::f32::consts::PI;

pub fn neighbors_set(
    (x, y): (i32, i32),
    (tl_x, tl_y): (i32, i32),
    (br_x, br_y): (i32, i32),
) -> HashSet<(i32, i32)> {
    let mut v = HashSet::new();

    if x > tl_x {
        v.insert((x - 1, y));

        if y > tl_y {
            v.insert((x - 1, y - 1));
        }

        if y < br_y {
            v.insert((x - 1, y + 1));
        }
    }

    if x < br_x {
        v.insert((x + 1, y));

        if y > tl_y {
            v.insert((x + 1, y - 1));
        }

        if y < br_y {
            v.insert((x + 1, y + 1));
        }
    }

    if y > tl_y {
        v.insert((x, y - 1));
    }

    if y < br_y {
        v.insert((x, y + 1));
    }

    v
}

pub fn neighbors(
    (x, y): (i32, i32),
    (tl_x, tl_y): (i32, i32),
    (br_x, br_y): (i32, i32),
) -> Vec<(i32, i32)> {
    let mut v = Vec::new();

    if x > tl_x {
        v.push((x - 1, y));

        if y > tl_y {
            v.push((x - 1, y - 1));
        }

        if y < br_y {
            v.push((x - 1, y + 1));
        }
    }

    if x < br_x {
        v.push((x + 1, y));

        if y > tl_y {
            v.push((x + 1, y - 1));
        }

        if y < br_y {
            v.push((x + 1, y + 1));
        }
    }

    if y > tl_y {
        v.push((x, y - 1));
    }

    if y < br_y {
        v.push((x, y + 1));
    }

    v
}

pub fn magnitude((x, y): (f32, f32)) -> f32 {
    return ((x * x) + (y * y)).sqrt();
}

pub fn normalize((x, y): (f32, f32)) -> (f32, f32) {
    let mut norm = (x, y);
    norm.0 /= magnitude((x, y));
    norm.1 /= magnitude((x, y));
    norm
}

pub fn dot((x, y): (f32, f32), (w, z): (f32, f32)) -> f32 {
    x * w + y * z
}

pub fn discrete_jmp((x, y): (f32, f32)) -> (i32, i32) {
    let min = 0.01;
    let mut unit_x: i32 = 0;
    if x > 0.0 && x - min > 0.0 {
        unit_x = 1;
    } else if x < 0.0 && x + min < 0.0 {
        unit_x = -1;
    }

    let mut unit_y: i32 = 0;
    if y > 0.0 && y - min > 0.0 {
        unit_y = 1;
    } else if y < 0.0 && y + min < 0.0 {
        unit_y = -1;
    }

    (unit_x, unit_y)
}

pub fn rotate_left((x, y): (i32, i32)) -> (i32, i32) {
    let angle = PI / 8.0;
    rotate((x, y), angle)
}

pub fn rotate_right((x, y): (i32, i32)) -> (i32, i32) {
    let angle = 2.0 * PI - (PI / 8.0);
    rotate((x, y), angle)
}

pub fn rotate((x, y): (i32, i32), angle: f32) -> (i32, i32) {
    let x = x as f32;
    let y = y as f32;
    let new_x = (x * angle.cos()) - (y * angle.cos());
    let new_y = (x * angle.sin()) + (y * angle.sin());
    let min = 0.01;
    let ix: i32;
    let iy: i32;
    if new_x.abs() < min {
        ix = 0;
    } else {
        ix = new_x.abs().ceil() as i32 * new_x.signum() as i32;
    }
    if new_y.abs() < min {
        iy = 0;
    } else {
        iy = new_y.abs().ceil() as i32 * new_y.signum() as i32;
    }

    (ix.clamp(-1, 1), iy.clamp(-1, 1))
}
