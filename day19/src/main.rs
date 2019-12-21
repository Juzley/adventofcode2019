use intcode::Program;

const SQUARE_SIZE: i64 = 100;

enum Result {
    Big,
    Small,
    Fits(i64, i64),
}

fn is_tractor_beam(prg: &Program, x: i64, y: i64) -> bool {
    let input = vec![x, y];
    let mut iter = input.iter();
    let mut prg = prg.clone();
    let mut result = false;
    let mut awaiting_output = true;
    while awaiting_output {
        let _ = prg.step(&mut || *iter.next().unwrap(), &mut |v| {
            if v > 0 {
                result = true;
            }
            awaiting_output = false;
        });
    }

    result
}

fn find_row_bounds(prg: &Program, y: i64) -> (i64, i64) {
    let mut bounds = (None, None);
    let mut x = 0;
    while bounds.1.is_none() {
        if is_tractor_beam(prg, x, y) {
            if bounds.0.is_none() {
                bounds.0 = Some(x);
            }
        } else {
            if bounds.0.is_some() {
                bounds.1 = Some(x - 1);
                break;
            }
        }

        x += 1;
    }

    (bounds.0.unwrap(), bounds.1.unwrap())
}

fn square_fits(prg: &Program, y: i64) -> Result {
    println!("Trying row {}", y);

    let bounds = find_row_bounds(prg, y);

    if bounds.1 - bounds.0 < (SQUARE_SIZE - 2) {
        return Result::Small;
    }

    let left = bounds.1 - (SQUARE_SIZE - 1);
    let bottom = y + (SQUARE_SIZE - 1);

    let prev_in_beam = is_tractor_beam(prg, left - 1, bottom);
    let cur_in_beam = is_tractor_beam(prg, left, bottom);

    if prev_in_beam && cur_in_beam {
        return Result::Big;
    } else if !prev_in_beam && cur_in_beam {
        return Result::Fits(left, y);
    } else {
        return Result::Small;
    }
}

fn main() {
    let mut prg = Program::from_file("input");

    let mut lower = 10;
    let mut current = lower;
    let mut upper = lower;
    let mut result = None;

    // Find an upper bound
    loop {
        match square_fits(&prg, current) {
            Result::Small => {
                lower = current;
                current *= 2;
            }
            Result::Big => {
                upper = current;
                break;
            }
            Result::Fits(x, y) => {
                result = Some((x, y));
                break;
            }
        }
    }

    println!("Bounds: ({}, {})", lower, upper);

    // Binary search
    while result.is_none() {
        current = lower + (upper - lower) / 2;

        match square_fits(&prg, current) {
            Result::Small => {
                lower = current;
            }
            Result::Big => {
                upper = current;
            }
            Result::Fits(x, y) => {
                result = Some((x, y));
            }
        }
    }

    let result = result.unwrap();
    println!(
        "Closest point: ({}, {}). Result: {}",
        result.0,
        result.1,
        result.0 * 10000 + result.1
    );
}
