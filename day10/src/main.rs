use num_integer;
use std::collections::HashSet;
use std::f64;
use std::fs::File;
use std::io::{BufRead, BufReader};

const ASTEROID_CHAR: char = '#';
const TARGET_VAPORIZE_COUNT: usize = 200;

#[derive(Clone, Debug)]
struct Map {
    // Set of asteroid coordinates.
    asteroids: HashSet<(i32, i32)>,
}

impl Map {
    fn from_strings(input: &[String]) -> Map {
        let mut asteroids = HashSet::new();

        for y in 0..input.len() {
            input[y]
                .chars()
                .enumerate()
                .filter_map(|(x, c)| match c {
                    ASTEROID_CHAR => Some(x),
                    _ => None,
                })
                .for_each(|x| {
                    asteroids.insert((x as i32, y as i32));
                });
        }

        return Map {
            asteroids: asteroids,
        };
    }

    fn from_file(filename: &str) -> Map {
        let file = File::open(filename).expect("Failed to open file");
        let reader = BufReader::new(file);

        let result: Result<Vec<String>, _> = reader.lines().collect();
        let input = result.expect("Failed to read lines");
        return Map::from_strings(&input);
    }

    fn find_visible_asteroids(&self, src: (i32, i32)) -> Vec<(i32, i32)> {
        // Brute-force: loop through all asteroids and determine if
        // we can see this asteroid by checking for other asteroids
        // that block it.
        let mut asteroids = Vec::new();
        for tgt in &self.asteroids {
            if src == *tgt {
                continue;
            }

            let (tx, ty) = *tgt;

            // Find the step size along the line from the source to
            // the target asteroid that will land exactly on map
            // coordinates that we can use to check for asteroids.
            let gcd = num_integer::gcd(tx - src.0, ty - src.1);
            let step = ((tx - src.0) / gcd, (ty - src.1) / gcd);

            // Check for blocking asteroids along the line from
            // source to target.
            let mut blocked = false;
            let mut cur_location = (src.0 + step.0, src.1 + step.1);
            while cur_location != (tx, ty) {
                if self.asteroids.contains(&cur_location) {
                    blocked = true;
                    break;
                }

                cur_location = (cur_location.0 + step.0, cur_location.1 + step.1);
            }

            if !blocked {
                asteroids.push(*tgt);
            }
        }

        return asteroids;
    }

    fn vaporize_asteroids(&mut self, asteroids: &[(i32, i32)]) {
        for location in asteroids {
            self.asteroids.remove(location);
        }
    }
}

// Find the location on a map that can see the most asteroids.
// Return that location, plus the number of asteroids that can be
// seen from it.
fn find_optimal_monitoring_location(map: &Map) -> ((i32, i32), u32) {
    let mut max_asteroids = 0;
    let mut best_space = (0, 0);

    for src in &map.asteroids {
        let asteroids = map.find_visible_asteroids(*src);
        if asteroids.len() > max_asteroids {
            max_asteroids = asteroids.len();
            best_space = *src;
        }
    }

    return (best_space, max_asteroids as u32);
}

fn find_bearing(src: (i32, i32), dst: (i32, i32)) -> f64 {
    let theta = ((dst.0 - src.0) as f64).atan2((src.1 - dst.1) as f64);
    if theta < 0.0 {
        return theta + f64::consts::PI * 2.0;
    }
    return theta;
}

fn find_nth_vaporized(m: &Map, laser_loc: (i32, i32), n: usize) -> (i32, i32) {
    let mut vaporized = 0;
    let mut map = m.clone();

    assert!(n > 0);
    let n = n - 1;

    loop {
        let mut asteroids = map.find_visible_asteroids(laser_loc);
        if asteroids.is_empty() {
            panic!("No visible asteroids!");
        }

        if asteroids.len() + vaporized <= n {
            // We vaporize all these asteroids without reaching the count
            map.vaporize_asteroids(&asteroids);
            vaporized += asteroids.len();
        } else {
            // We only get part way through these asteroids, sort by the
            // order the laser will hit each one.
            asteroids.sort_by(|a, b| {
                find_bearing(laser_loc, *a)
                    .partial_cmp(&find_bearing(laser_loc, *b))
                    .unwrap()
            });

            return asteroids[n - vaporized];
        }
    }
}

fn main() {
    // Part 1
    let map = Map::from_file("input");
    let (coords, count) = find_optimal_monitoring_location(&map);
    println!("Best location {:?} sees {} asteroids", coords, count);

    // Part 2
    let result = find_nth_vaporized(&map, coords, TARGET_VAPORIZE_COUNT);
    println!(
        "Vaporized asteroid number {}: {:?}. Answer {}",
        TARGET_VAPORIZE_COUNT,
        result,
        result.0 * 100 + result.1
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearing_test() {
        let diff = (find_bearing((0, 1), (0, 0)) - 0.0).abs();
        assert!(diff < 1e-10);

        let diff = (find_bearing((0, 0), (1, 0)) - std::f64::consts::FRAC_PI_2).abs();
        assert!(diff < 1e-10);

        let diff = (find_bearing((0, 0), (0, 1)) - std::f64::consts::PI).abs();
        assert!(diff < 1e-10);

        let diff = (find_bearing((1, 0), (0, 0)) - std::f64::consts::FRAC_PI_2 * 3.0).abs();
        assert!(diff < 1e-10);
    }

    #[test]
    fn pt1_example_1() {
        let strs = vec![
            String::from(".#..#"),
            String::from("....."),
            String::from("#####"),
            String::from("....#"),
            String::from("...##"),
        ];
        let map = Map::from_strings(&strs);
        let (coords, count) = find_optimal_monitoring_location(&map);
        assert_eq!(coords, (3, 4));
        assert_eq!(count, 8);
    }

    #[test]
    fn pt1_example_2() {
        let strs = vec![
            String::from("......#.#."),
            String::from("#..#.#...."),
            String::from("..#######."),
            String::from(".#.#.###.."),
            String::from(".#..#....."),
            String::from("..#....#.#"),
            String::from("#..#....#."),
            String::from(".##.#..###"),
            String::from("##...#..#."),
            String::from(".#....####"),
        ];
        let map = Map::from_strings(&strs);
        let (coords, count) = find_optimal_monitoring_location(&map);
        assert_eq!(coords, (5, 8));
        assert_eq!(count, 33);
    }

    #[test]
    fn pt1_example_3() {
        let strs = vec![
            String::from("#.#...#.#."),
            String::from(".###....#."),
            String::from(".#....#..."),
            String::from("##.#.#.#.#"),
            String::from("....#.#.#."),
            String::from(".##..###.#"),
            String::from("..#...##.."),
            String::from("..##....##"),
            String::from("......#..."),
            String::from(".####.###."),
        ];
        let map = Map::from_strings(&strs);
        let (coords, count) = find_optimal_monitoring_location(&map);
        assert_eq!(coords, (1, 2));
        assert_eq!(count, 35);
    }

    #[test]
    fn pt1_example_4() {
        let strs = vec![
            String::from(".#..#..###"),
            String::from("####.###.#"),
            String::from("....###.#."),
            String::from("..###.##.#"),
            String::from("##.##.#.#."),
            String::from("....###..#"),
            String::from("..#.#..#.#"),
            String::from("#..#.#.###"),
            String::from(".##...##.#"),
            String::from(".....#.#.."),
        ];
        let map = Map::from_strings(&strs);
        let (coords, count) = find_optimal_monitoring_location(&map);
        assert_eq!(coords, (6, 3));
        assert_eq!(count, 41);
    }

    #[test]
    fn pt1_example_5() {
        let strs = vec![
            String::from(".#..##.###...#######"),
            String::from("##.############..##."),
            String::from(".#.######.########.#"),
            String::from(".###.#######.####.#."),
            String::from("#####.##.#.##.###.##"),
            String::from("..#####..#.#########"),
            String::from("####################"),
            String::from("#.####....###.#.#.##"),
            String::from("##.#################"),
            String::from("#####.##.###..####.."),
            String::from("..######..##.#######"),
            String::from("####.##.####...##..#"),
            String::from(".#####..#.######.###"),
            String::from("##...#.##########..."),
            String::from("#.##########.#######"),
            String::from(".####.#.###.###.#.##"),
            String::from("....##.##.###..#####"),
            String::from(".#.#.###########.###"),
            String::from("#.#.#.#####.####.###"),
            String::from("###.##.####.##.#..##"),
        ];
        let map = Map::from_strings(&strs);
        let (coords, count) = find_optimal_monitoring_location(&map);
        assert_eq!(coords, (11, 13));
        assert_eq!(count, 210);
    }

    #[test]
    fn pt2_example_1() {
        let strs = vec![
            String::from(".#....#####...#.."),
            String::from("##...##.#####..##"),
            String::from("##...#...#.#####."),
            String::from("..#.....X...###.."),
            String::from("..#.#.....#....##"),
        ];
        let map = Map::from_strings(&strs);
        println!("{}", map.asteroids.len());
        let station_coords = (8, 3);

        let tests = vec![(9, (15, 1)), (18, (4, 4)), (27, (5, 1)), (36, (14, 3))];

        for (n, exp_coords) in tests {
            let coords = find_nth_vaporized(&map, station_coords, n);
            assert_eq!(coords, exp_coords);
        }
    }

    #[test]
    fn pt2_example_2() {
        let strs = vec![
            String::from(".#..##.###...#######"),
            String::from("##.############..##."),
            String::from(".#.######.########.#"),
            String::from(".###.#######.####.#."),
            String::from("#####.##.#.##.###.##"),
            String::from("..#####..#.#########"),
            String::from("####################"),
            String::from("#.####....###.#.#.##"),
            String::from("##.#################"),
            String::from("#####.##.###..####.."),
            String::from("..######..##.#######"),
            String::from("####.##.####...##..#"),
            String::from(".#####..#.######.###"),
            String::from("##...#.##########..."),
            String::from("#.##########.#######"),
            String::from(".####.#.###.###.#.##"),
            String::from("....##.##.###..#####"),
            String::from(".#.#.###########.###"),
            String::from("#.#.#.#####.####.###"),
            String::from("###.##.####.##.#..##"),
        ];

        let map = Map::from_strings(&strs);
        let (station_coords, count) = find_optimal_monitoring_location(&map);

        let tests = vec![
            (1, (11, 12)),
            (2, (12, 1)),
            (3, (12, 2)),
            (10, (12, 8)),
            (20, (16, 0)),
            (50, (16, 9)),
            (100, (10, 16)),
            (199, (9, 6)),
            (200, (8, 2)),
            (201, (10, 9)),
            (299, (11, 1)),
        ];

        for (n, exp_coords) in tests {
            let coords = find_nth_vaporized(&map, station_coords, n);
            assert_eq!(coords, exp_coords);
        }
    }
}
