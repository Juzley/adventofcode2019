use intcode::Program;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pathfinding::prelude::{absdiff, astar};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(i64)]
enum Direction {
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

#[derive(Copy, Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(i64)]
enum LocType {
    Empty = 1,
    Wall = 0,
    Oxygen = 2,
}

type Loc = (i64, i64);
type Map = HashMap<Loc, LocType>;

// Get the direction between two neighbouring locations. Panics
// if the tiles aren't neighbouring.
fn get_direction(start: Loc, end: Loc) -> Direction {
    if end.0 - start.0 == 1 && end.1 == start.1 {
        return Direction::East;
    } else if start.0 - end.0 == 1 && end.1 == start.1 {
        return Direction::West;
    } else if start.0 == end.0 && end.1 - start.1 == 1 {
        return Direction::North;
    } else if start.0 == end.0 && start.1 - end.1 == 1 {
        return Direction::South;
    }

    panic!("Can't get direction between non-neighbouring tiles");
}

fn get_neighbour_coords(loc: Loc) -> Vec<Loc> {
    vec![
        (loc.0 + 1, loc.1),
        (loc.0 - 1, loc.1),
        (loc.0, loc.1 + 1),
        (loc.0, loc.1 - 1),
    ]
}

// Find a path between two locations on a given map. Assumes a path
// exists, panics otherwise.
fn find_path(start: Loc, goal: Loc, map: &Map) -> Vec<Loc> {
    let distance = |&loc: &Loc| (absdiff(loc.0, goal.0) + absdiff(loc.1, goal.1)) as u64;

    let successors = |&loc: &Loc| -> Vec<(Loc, u64)> {
        get_neighbour_coords(loc)
            .into_iter()
            .filter(|candidate: &Loc| match map.get(candidate) {
                Some(LocType::Empty) => true,
                Some(LocType::Oxygen) => true,
                _ => false,
            })
            .map(|loc| (loc, 1))
            .collect()
    };

    return astar(&start, successors, distance, |&loc| loc == goal)
        .map(|tuple| tuple.0)
        .unwrap();
}

// Attempt to step the robot in a given direction and return
// the resulting location type.
fn step_one(dir: Direction, robot: &mut Program) -> LocType {
    let mut out: Option<LocType> = None;
    while out.is_none() {
        let _ = robot.step(&mut || dir.into(), &mut |val| {
            out = Some(LocType::try_from(val).unwrap())
        });
    }
    out.unwrap()
}

// Move the robot along a given path from a given start position.
// It is assume the path has already been explored and has no walls.
fn follow_path(start: Loc, path: &Vec<Loc>, robot: &mut Program) {
    let mut current = start;

    for loc in path {
        if current != *loc {
            let dir = get_direction(current, *loc);
            let loc_type = step_one(dir, robot);
            assert!(loc_type != LocType::Wall);
            current = *loc;
        }
    }
}

// Move the robot to a given point. It is assumed that a path between the
// start and goal exists in the given map.
fn navigate_to(start: Loc, goal: Loc, map: &Map, robot: &mut Program) {
    if start != goal {
        let path = find_path(start, goal, map);
        follow_path(start, &path, robot);
    }
}

// Explore any unexplored neighbouring tiles, update the map and return the list of
// newly explored tiles that can be visited (i.e. are not walls).
fn explore_neighbours(loc: Loc, map: &mut Map, robot: &mut Program) -> VecDeque<(Loc, LocType)> {
    let mut result = VecDeque::new();

    for neighbour in get_neighbour_coords(loc) {
        if map.contains_key(&neighbour) {
            continue;
        }

        let dir = get_direction(loc, neighbour);
        let loc_type = step_one(dir, robot);

        // If we hit a wall, we haven't moved anywhere and can continue
        // immediately, otherwise we need to move back to the start
        // square.
        match loc_type {
            LocType::Wall => continue,
            _ => {
                map.insert(neighbour, loc_type);
                result.push_back((neighbour, loc_type));

                let dir = get_direction(neighbour, loc);
                let loc_type = step_one(dir, robot);
                assert!(loc_type != LocType::Wall);
            }
        }
    }

    result
}

// Generates a fully-explored map, and the location of the oxygen, relative to the
// start location.
fn explore(robot: &mut Program) -> (Map, Loc) {
    let mut current_loc = (0, 0);
    let mut loc_queue = VecDeque::new();
    loc_queue.push_back(current_loc);

    let mut map = HashMap::new();
    let mut oxygen = None;
    while !loc_queue.is_empty() {
        let next_loc = loc_queue.pop_front().unwrap();
        if current_loc != next_loc {
            navigate_to(current_loc, next_loc, &map, robot);
            current_loc = next_loc;
        }

        let new_locs = explore_neighbours(current_loc, &mut map, robot);
        for (loc, loc_type) in new_locs {
            // Check whether we found the oxygen.
            if loc_type == LocType::Oxygen {
                oxygen = Some(loc);
            }
            loc_queue.push_back(loc);
        }
    }

    (map, oxygen.unwrap())
}

fn fill_oxygen(start: Loc, map: &mut Map) -> u64 {
    let mut current_locs = vec![start];
    let mut minutes = 0;

    loop {
        let mut next_locs = Vec::new();
        for loc in current_locs {
            for neighbour in get_neighbour_coords(loc) {
                if let Some(loc_type) = map.get_mut(&neighbour) {
                    match *loc_type {
                        LocType::Empty => {
                            next_locs.push(neighbour);
                            *loc_type = LocType::Oxygen;
                        }
                        _ => (),
                    }
                }
            }
        }

        if next_locs.is_empty() {
            break;
        }

        current_locs = next_locs;
        minutes += 1;
    }

    minutes
}

fn main() {
    let mut robot = Program::from_file("input");
    let (map, oxygen) = explore(&mut robot);

    // Part 1
    let path = find_path((0, 0), oxygen, &map);
    println!("Robot needs {} steps to get to the oxygen", path.len() - 1);

    // Part 2
    let minutes = fill_oxygen(oxygen, &mut map.clone());
    println!("Area fills with oxygen in {} minutes", minutes);
}
