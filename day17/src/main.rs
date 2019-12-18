use intcode::Program;
use std::ops::Range;

const MAX_BUF_LEN: usize = 20;

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Down,
    Up,
}

#[derive(Clone, Copy)]
enum TileType {
    Scaffold,
    Space,
    Robot(Direction),
    RobotFalling,
}

impl TileType {
    fn from_ascii(ascii: i64) -> TileType {
        match ascii {
            35 => TileType::Scaffold,
            46 => TileType::Space,
            60 => TileType::Robot(Direction::Left),
            62 => TileType::Robot(Direction::Right),
            76 => TileType::Robot(Direction::Down),
            88 => TileType::RobotFalling,
            94 => TileType::Robot(Direction::Up),
            _ => panic!("Unrecognized ascii code"),
        }
    }

    fn to_ascii(tile: Self) -> u8 {
        match tile {
            TileType::Scaffold => 35,
            TileType::Space => 46,
            TileType::Robot(Direction::Left) => 60,
            TileType::Robot(Direction::Right) => 62,
            TileType::Robot(Direction::Down) => 76,
            TileType::RobotFalling => 88,
            TileType::Robot(Direction::Up) => 94,
        }
    }

    fn is_scaffold(tile: Self) -> bool {
        match tile {
            TileType::Space => false,
            TileType::RobotFalling => false,
            _ => true,
        }
    }

    fn is_robot_on_scaffold(tile: Self) -> bool {
        match tile {
            TileType::Robot(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    TurnRight,
    TurnLeft,
    Move(usize),
}

impl Command {
    fn to_string(self: Self) -> String {
        match self {
            Command::TurnRight => String::from("R"),
            Command::TurnLeft => String::from("L"),
            Command::Move(d) => d.to_string(),
        }
    }
}

type Map = Vec<Vec<TileType>>;
type Coords = (usize, usize);

fn get_map(program: &Program) -> Map {
    let mut map = Vec::new();
    let mut row = Vec::new();
    program.execute_ex(
        || 0,
        |val| {
            // Hit a newline, start a new row
            if val == 10 {
                if !row.is_empty() {
                    map.push(row.clone());
                    row.clear();
                }
            } else {
                row.push(TileType::from_ascii(val));
            }
        },
    );

    map
}

fn get_neighbour_coords(map: &Map, coords: Coords) -> Vec<Coords> {
    let mut nbrs = Vec::new();
    if coords.0 > 0 {
        nbrs.push((coords.0 - 1, coords.1));
    }
    if coords.0 < map[0].len() - 1 {
        nbrs.push((coords.0 + 1, coords.1));
    }
    if coords.1 > 0 {
        nbrs.push((coords.0, coords.1 - 1));
    }
    if coords.1 < map.len() - 1 {
        nbrs.push((coords.0, coords.1 + 1));
    }

    nbrs
}

fn find_intersections(map: &Map) -> Vec<Coords> {
    let mut intersections = Vec::new();

    for (y, row) in map.iter().enumerate() {
        for (x, &tile) in row.iter().enumerate() {
            if !TileType::is_scaffold(tile) {
                continue;
            }

            let scaffold_neighbours = get_neighbour_coords(map, (x, y))
                .iter()
                .map(|&(nx, ny)| map[ny][nx])
                .filter(|&tile| TileType::is_scaffold(tile))
                .count();
            if scaffold_neighbours == 4 {
                intersections.push((x, y));
            }
        }
    }

    return intersections;
}

fn find_vacuum(map: &Map) -> Coords {
    for (y, row) in map.iter().enumerate() {
        for (x, &tile) in row.iter().enumerate() {
            if TileType::is_robot_on_scaffold(tile) {
                return (x, y);
            }
        }
    }

    panic!("Couldn't find robot");
}

fn print_map(map: &Map) {
    for row in map {
        println!(
            "{}",
            row.iter()
                .map(|&t| TileType::to_ascii(t) as char)
                .collect::<String>(),
        )
    }
}

fn find_next_direction(map: &Map, dir: Direction, coords: Coords) -> Option<(Command, Direction)> {
    match dir {
        Direction::Left => {
            if coords.1 > 0 && TileType::is_scaffold(map[coords.1 - 1][coords.0]) {
                return Some((Command::TurnRight, Direction::Up));
            }
            if coords.1 < map.len() - 1 && TileType::is_scaffold(map[coords.1 + 1][coords.0]) {
                return Some((Command::TurnLeft, Direction::Down));
            }
        }
        Direction::Right => {
            if coords.1 > 0 && TileType::is_scaffold(map[coords.1 - 1][coords.0]) {
                return Some((Command::TurnLeft, Direction::Up));
            }
            if coords.1 < map.len() - 1 && TileType::is_scaffold(map[coords.1 + 1][coords.0]) {
                return Some((Command::TurnRight, Direction::Down));
            }
        }
        Direction::Up => {
            if coords.0 > 0 && TileType::is_scaffold(map[coords.1][coords.0 - 1]) {
                return Some((Command::TurnLeft, Direction::Left));
            }
            if coords.0 < map[0].len() - 1 && TileType::is_scaffold(map[coords.1][coords.0 + 1]) {
                return Some((Command::TurnRight, Direction::Right));
            }
        }
        Direction::Down => {
            if coords.0 > 0 && TileType::is_scaffold(map[coords.1][coords.0 - 1]) {
                return Some((Command::TurnRight, Direction::Left));
            }
            if coords.0 < map[0].len() - 1 && TileType::is_scaffold(map[coords.1][coords.0 + 1]) {
                return Some((Command::TurnLeft, Direction::Right));
            }
        }
    }

    // Didn't find a way to go, must be finished.
    None
}

fn gen_move(map: &Map, mut coords: Coords, dir: Direction) -> (usize, Coords) {
    let mut distance = 0;

    match dir {
        Direction::Up => {
            while coords.1 > 0 && TileType::is_scaffold(map[coords.1 - 1][coords.0]) {
                distance += 1;
                coords = (coords.0, coords.1 - 1);
            }
        }
        Direction::Down => {
            while coords.1 < map.len() - 1 && TileType::is_scaffold(map[coords.1 + 1][coords.0]) {
                distance += 1;
                coords = (coords.0, coords.1 + 1);
            }
        }
        Direction::Left => {
            while coords.0 > 0 && TileType::is_scaffold(map[coords.1][coords.0 - 1]) {
                distance += 1;
                coords = (coords.0 - 1, coords.1);
            }
        }
        Direction::Right => {
            while coords.0 < map[0].len() - 1 && TileType::is_scaffold(map[coords.1][coords.0 + 1])
            {
                distance += 1;
                coords = (coords.0 + 1, coords.1);
            }
        }
    }

    (distance, coords)
}

fn gen_path(map: &Map, start: Coords) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut current_coords = start;

    // Find the direction the robot is facing at the start.
    let mut current_dir = match map[current_coords.1][current_coords.0] {
        TileType::Robot(dir) => dir,
        _ => panic!("Robot isn't at start coords"),
    };

    loop {
        if let Some((cmd, new_dir)) = find_next_direction(map, current_dir, current_coords) {
            // We found a new direction to go in, generate commands to turn and then move as far as
            // we can in that direction.
            commands.push(cmd);
            current_dir = new_dir;

            let (distance, new_coords) = gen_move(map, current_coords, new_dir);
            commands.push(Command::Move(distance));
            current_coords = new_coords;
        } else {
            // No new direction found, we must have finished.
            break;
        }
    }

    commands
}

// Calculate the length of a function.
fn function_len(commands: &[Command]) -> usize {
    let len: usize = commands
        .iter()
        .map(|&c| match c {
            Command::TurnLeft | Command::TurnRight => 1,
            Command::Move(d) => d.to_string().len(),
        })
        .sum();

    len + commands.len() - 1
}

// Find the first position in the given command buffer that isn't covered by any of
// the supplied functions.
fn skip_functions(commands: &Vec<Command>, mut start: usize, functions: Vec<&[Command]>) -> usize {
    loop {
        let mut found = false;
        for f in &functions {
            let slice = &commands[start..(start + f.len())];
            if *slice == **f {
                found = true;
                start += f.len();
                break;
            }
        }

        if !found {
            return start;
        }
    }
}

// Checks whether the given set of functions executed the required commands.
fn check_functions(commands: &Vec<Command>, functions: &Vec<&[Command]>) -> Option<Vec<usize>> {
    let mut routine = Vec::new();
    let mut start = 0;

    loop {
        let mut found = false;
        for (i, f) in functions.iter().enumerate() {
            let slice = &commands[start..(start + f.len())];
            if slice == *f {
                found = true;
                start += f.len();
                routine.push(i);
                break;
            }
        }

        // Didn't find a match for any of the functions, not a valid set.
        if !found {
            return None;
        }

        // Used all the input exactly, this is a valid set of functions.
        if start == commands.len() {
            return Some(routine);
        }

        // Went too far, not a valid set.
        if start > commands.len() {
            return None;
        }
    }
}

fn get_candidate_function<'a>(
    commands: &'a Vec<Command>,
    range: &Range<usize>,
) -> Option<&'a [Command]> {
    if range.end >= commands.len() - 1 {
        return None;
    }
    let function = &commands[range.clone()];
    if function_len(function) > MAX_BUF_LEN {
        return None;
    }

    Some(function)
}

fn find_movement_routine(commands: &Vec<Command>) -> (Vec<usize>, Vec<&[Command]>) {
    let mut a_range = 0..1;

    loop {
        // Loop over possible A functions
        let candidate = get_candidate_function(commands, &a_range);
        if candidate.is_none() {
            break;
        }
        let a_function = candidate.unwrap();
        let b_start = skip_functions(commands, a_range.end, vec![a_function]);
        let mut b_range = b_start..(b_start + 1);

        loop {
            // Loop over possible B functions, given the current A function
            let candidate = get_candidate_function(commands, &b_range);
            if candidate.is_none() {
                break;
            }
            let b_function = candidate.unwrap();
            let c_start = skip_functions(commands, b_range.end, vec![a_function, b_function]);
            let mut c_range = c_start..(c_start + 1);

            loop {
                // Loop over possible C functions, given the current A and B functions
                let candidate = get_candidate_function(commands, &c_range);
                if candidate.is_none() {
                    break;
                }
                let c_function = candidate.unwrap();

                let fns = vec![a_function, b_function, c_function];

                if let Some(fn_indices) = check_functions(commands, &fns) {
                    return (fn_indices, fns);
                }

                c_range.end += 1;
            }

            b_range.end += 1;
        }

        a_range.end += 1;
    }

    panic!("Failed to find possible functions");
}

// Convert the movement routine, in the form of a vector of functions, and a vector
// of indices into the function vector representing the order in which to execute
// those functions, into a vector of ascii chars to provide as input to the robot
// program.
fn make_robot_input(routine: &Vec<usize>, functions: &Vec<&[Command]>) -> Vec<u8> {
    let routine_str = routine
        .iter()
        .map(|&i| match i {
            0 => String::from("A"),
            1 => String::from("B"),
            2 => String::from("C"),
            _ => panic!("Unexpected function index"),
        })
        .collect::<Vec<String>>()
        .join(",");

    let function_strs = functions
        .iter()
        .map(|f| {
            f.iter()
                .map(|cmd| cmd.to_string())
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>();

    // Disable video output.
    let video_str = String::from("n");

    let mut input_strs: Vec<String> = vec![routine_str];
    input_strs.extend(function_strs);
    input_strs.push(video_str);

    let input_strs = input_strs
        .into_iter()
        .map(|s| s + "\n")
        .collect::<Vec<String>>();
    input_strs.join("").chars().map(|c| c as u8).collect()
}

// Move the vacuum robot by executing the given movement routine input.
fn move_robot(program: &Program, input: &Vec<u8>) -> i64 {
    let mut program = program.clone();
    program.poke(0, 2);

    let mut input_iter = input.iter();
    let mut output = None;

    program.execute_ex(|| *input_iter.next().unwrap() as i64, |v| output = Some(v));
    output.unwrap()
}

fn main() {
    let program = Program::from_file("input");

    let map = get_map(&program);
    print_map(&map);

    // Part 1
    let intersections = find_intersections(&map);
    let result = intersections.iter().map(|(x, y)| x * y).sum::<usize>();
    println!("Intersection Sum: {}", result);

    // Part 2
    let vacuum_coords = find_vacuum(&map);
    let commands = gen_path(&map, vacuum_coords);
    println!("Commands: {:?}", commands);
    let (routine, functions) = find_movement_routine(&commands);
    println!("Routine: {:?}, Functions: {:?}", routine, functions);
    let program_input = make_robot_input(&routine, &functions);
    println!("{:?}", program_input);
    let result = move_robot(&program, &program_input);
    println!("Vacuumed {} dust", result);
}
