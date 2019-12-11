extern crate intcode;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

const BLACK: u8 = 0;
const WHITE: u8 = 1;

const ROTATE_RIGHT: i64 = 1;
const ROTATE_LEFT: i64 = 0;

#[derive(Clone, Copy)]
enum Direction {
    UP,
    RIGHT,
    DOWN,
    LEFT,
}

enum Mode {
    PAINT,
    ROTATE,
}

// Returns the painting robot, returns a map from the coordinates it painted to
// the colour it painted them.
fn run_paint_robot(filename: &str, start_color: u8) -> HashMap<(i64, i64), u8> {
    let current_coords = Cell::new((0, 0));
    let hull: RefCell<HashMap<(i64, i64), u8>> = RefCell::new(HashMap::new());
    let mut mode = Mode::PAINT;
    let mut dir = Direction::UP;

    hull.borrow_mut().insert(current_coords.get(), start_color);

    let input_fn = || {
        let h = hull.borrow();
        let coords = current_coords.get();
        if h.contains_key(&coords) {
            return *h.get(&coords).unwrap() as i64;
        } else {
            return BLACK as i64;
        }
    };

    let output_fn = |val| match mode {
        Mode::PAINT => {
            let mut h = hull.borrow_mut();
            h.insert(current_coords.get(), val as u8);
            mode = Mode::ROTATE;
        }
        Mode::ROTATE => {
            dir = match (dir, val) {
                (Direction::UP, ROTATE_LEFT) => Direction::LEFT,
                (Direction::UP, ROTATE_RIGHT) => Direction::RIGHT,
                (Direction::RIGHT, ROTATE_LEFT) => Direction::UP,
                (Direction::RIGHT, ROTATE_RIGHT) => Direction::DOWN,
                (Direction::DOWN, ROTATE_LEFT) => Direction::RIGHT,
                (Direction::DOWN, ROTATE_RIGHT) => Direction::LEFT,
                (Direction::LEFT, ROTATE_LEFT) => Direction::DOWN,
                (Direction::LEFT, ROTATE_RIGHT) => Direction::UP,
                _ => panic!("Invalid direction, rotation combination"),
            };

            let mut coords = current_coords.get();
            match dir {
                Direction::UP => coords.1 -= 1,
                Direction::RIGHT => coords.0 += 1,
                Direction::DOWN => coords.1 += 1,
                Direction::LEFT => coords.0 -= 1,
            };
            current_coords.set(coords);

            mode = Mode::PAINT;
        }
    };

    let program = intcode::Program::from_file(filename);
    program.execute_ex(input_fn, output_fn);

    return hull.into_inner();
}

fn robot_output_to_file(output: &HashMap<(i64, i64), u8>, filename: &str) {
    // Find the bounds of the image
    let mut min_x: i64 = 0;
    let mut max_x: i64 = 0;
    let mut min_y: i64 = 0;
    let mut max_y: i64 = 0;
    for ((x, y), _) in output {
        min_x = std::cmp::min(*x, min_x);
        max_x = std::cmp::max(*x, max_x);
        min_y = std::cmp::min(*y, min_y);
        max_y = std::cmp::max(*y, max_y);
    }

    let width = (max_x - min_x) as u32;
    let height = (max_y - min_y) as u32;

    let mut buf = image::ImageBuffer::new(width + 1, height + 1);

    for (x, y, pixel) in buf.enumerate_pixels_mut() {
        // Adjust the coords to allow for the fact that the
        // robot output is from (min_x, min_y) to (max_x, max_y),
        // rather than (0, 0) to (width, height).
        let adjusted_x = x as i64 + min_x;
        let adjusted_y = y as i64 + min_y;

        let px_val = match output.get(&(adjusted_x, adjusted_y)) {
            None => image::Rgb([0; 3]),
            Some(c) => image::Rgb([c * 255; 3]),
        };
        *pixel = px_val;
    }
    buf.save(filename).unwrap();
}

fn main() {
    let robot_output = run_paint_robot("input", WHITE);
    robot_output_to_file(&robot_output, "output.png");
}
