use pathfinding::prelude::{absdiff, astar};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Coords = (usize, usize);

#[derive(Debug)]
enum Tile {
    Empty,
    Floor,
    Wall,
    Warp(Coords),
}

#[derive(Debug)]
struct Map {
    tiles: Vec<Vec<Tile>>,
    warps: Vec<Coords>,
    start: Coords,
    end: Coords,
}

impl Map {
    fn find_tile_labels(coords: Coords, lines: &Vec<Vec<char>>) -> Option<String> {
        let label_chars = 'A'..='Z';

        let first = lines[coords.1 - 2][coords.0];
        let second = lines[coords.1 - 1][coords.0];        
        if label_chars.contains(&first) && label_chars.contains(&second) {
            return Some(vec![first, second].iter().collect::<String>());
        }

        let first = lines[coords.1 + 1][coords.0];
        let second = lines[coords.1 + 2][coords.0];
        if label_chars.contains(&first) && label_chars.contains(&second) {
            return Some(vec![first, second].iter().collect::<String>());
        }

        let first = lines[coords.1][coords.0 - 2];
        let second = lines[coords.1][coords.0 - 1];
        if label_chars.contains(&first) && label_chars.contains(&second) {
            return Some(vec![first, second].iter().collect::<String>());
        }

        let first = lines[coords.1][coords.0 + 1];
        let second = lines[coords.1][coords.0 + 2];
        if label_chars.contains(&first) && label_chars.contains(&second) {
            return Some(vec![first, second].iter().collect::<String>());
        }

        None
    }

    fn from_lines(lines: &Vec<String>) -> Self {
        let mut warps: HashMap<String, Coords> = HashMap::new();
        let mut warps_vec = Vec::new();

        let mut start = None;
        let mut end = None;

        let mut tiles = Vec::new();
        let lines: Vec<Vec<char>> = lines
            .into_iter()
            .map(|l| l.chars().collect::<Vec<char>>())
            .collect();
        for (line_idx_y, line) in lines.iter().enumerate() {
            // Skip the border.
            if line_idx_y < 2 || line_idx_y >= lines.len() - 2 {
                continue;
            }

            tiles.push(Vec::new());

            for (line_idx_x, c) in line.iter().enumerate() {
                // Skip the border.
                if line_idx_x < 2 || line_idx_x >= line.len() - 2 {
                    continue;
                }

                // Adjust the coords to allow for the fact we're skipping the borders.
                let x = line_idx_x - 2;
                let y = line_idx_y - 2;
                let coords = (x, y);

                let tile = match c {
                    '#' => Tile::Wall,
                    ' ' => Tile::Empty,
                    '.' => {
                        // Floor tile, need to check whether this is a labelled tile.
                        let label = Map::find_tile_labels((line_idx_x, line_idx_y), &lines);
                        if label.is_some() {
                            let label = label.unwrap();
                            match label.as_ref() {
                                "AA" => {
                                    start = Some(coords);
                                    Tile::Floor
                                },
                                "ZZ" => {
                                    end = Some(coords);
                                    Tile::Floor
                                },
                                _ => {
                                    warps_vec.push(coords);
                                    if let Some(warp_coords) = warps.get(&label) {
                                        // We already have the other end of this warp,
                                        // update both tiles.
                                        tiles[warp_coords.1][warp_coords.0] = Tile::Warp(coords);
                                        Tile::Warp(warps[&label])
                                    } else {
                                        // We don't have the other end of this warp yet,
                                        // add it to the map so we can look it up later.
                                        warps.insert(label, coords);
                                        Tile::Floor
                                    }
                                },
                            }
                        } else {
                            Tile::Floor
                        }
                    },
                    _ => Tile::Empty,
                };
                tiles[y].push(tile);
            }
        }

        Map {
            tiles: tiles,
            warps: warps_vec,
            start: start.expect("Didn't find start tile"),
            end: end.expect("Didn't find end tile"),
        }
    }

    fn from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        Map::from_lines(&lines)
    }

    fn get_neighbours(&self, coords: Coords) -> Vec<Coords> {
        let mut neighbours: Vec<Coords> = Vec::new();

        // If this is a warp tile, add the other end as a neighbour.
        match self.tiles[coords.1][coords.0] {
            Tile::Warp(c) => neighbours.push(c),
            _ => (),
        }

        // Add "normal" neighbours.
        if coords.0 > 0 {
            neighbours.push((coords.0 - 1, coords.1));
        }
        if coords.0 < self.tiles[0].len() - 1 {
            neighbours.push((coords.0 + 1, coords.1));
        }
        if coords.1 > 0 {
            neighbours.push((coords.0, coords.1 - 1));
        }
        if coords.1 < self.tiles.len() - 1 {
            neighbours.push((coords.0, coords.1 + 1));
        }

        neighbours
            .iter()
            .cloned()
            .filter(|(x, y)|
                match self.tiles[*y][*x] {
                    Tile::Floor => true,
                    Tile::Warp(_) => true,
                    _ => false,
                }
            )
            .collect()
    }

    fn find_nearest_warp(&self, coords: Coords) -> Coords {
        *self.warps
            .iter()
            .map(|c| (c, absdiff(coords.0, c.0) + absdiff(coords.1, c.1)))
            .min_by(|(_, d1), (_, d2)| d1.cmp(d2))
            .unwrap()
            .0
    }

    fn find_path_len(&self) -> usize {
        let distance = |&coords: &Coords| {
            let mhd = |c1: Coords, c2: Coords| { absdiff(c1.0, c2.0) + absdiff(c1.1, c2.1) };

            // Find the distance to the goal from the nearest teleporter.
            let warp_start_coords = self.find_nearest_warp(coords);
            let warp_end_coords = match self.tiles[warp_start_coords.1][warp_start_coords.0] {
                Tile::Warp(c) => c,
                _ => panic!("Malformed warp"),
            };

            let warp_distance = mhd(coords, warp_start_coords) + mhd(warp_end_coords, self.end);
            let direct_distance = mhd(coords, self.end);

            // Use the minimum of the distance via the nearest teleporter, and the
            // distance without using a teleporter.
            std::cmp::min(direct_distance, warp_distance)
        };

        let successors = |&coords: &Coords| -> Vec<(Coords, usize)> {
            self.get_neighbours(coords)
                .into_iter()
                .map(|coords| (coords, 1))
                .collect()
        };

        return astar(&self.start, successors, distance, |&coords| coords == self.end)
            .map(|tup| tup.1)
            .unwrap()
    }
}

fn main() {
    let map = Map::from_file("input");
    let len = map.find_path_len();
    println!("Shortest Path: {:?}", len);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt1_ex1() {
        let map = Map::from_lines(&vec![
            String::from("         A           "),
            String::from("         A           "),
            String::from("  #######.#########  "),
            String::from("  #######.........#  "),
            String::from("  #######.#######.#  "),
            String::from("  #######.#######.#  "),
            String::from("  #######.#######.#  "),
            String::from("  #####  B    ###.#  "),
            String::from("BC...##  C    ###.#  "),
            String::from("  ##.##       ###.#  "),
            String::from("  ##...DE  F  ###.#  "),
            String::from("  #####    G  ###.#  "),
            String::from("  #########.#####.#  "),
            String::from("DE..#######...###.#  "),
            String::from("  #.#########.###.#  "),
            String::from("FG..#########.....#  "),
            String::from("  ###########.#####  "),
            String::from("             Z       "),
            String::from("             Z       "),
        ]);

        let len = map.find_path_len();
        assert_eq!(len, 23);
    }

    #[test]
    fn pt1_ex2() {
        let map = Map::from_lines(&vec![
            String::from("                   A               "),
            String::from("                   A               "),
            String::from("  #################.#############  "),
            String::from("  #.#...#...................#.#.#  "),
            String::from("  #.#.#.###.###.###.#########.#.#  "),
            String::from("  #.#.#.......#...#.....#.#.#...#  "),
            String::from("  #.#########.###.#####.#.#.###.#  "),
            String::from("  #.............#.#.....#.......#  "),
            String::from("  ###.###########.###.#####.#.#.#  "),
            String::from("  #.....#        A   C    #.#.#.#  "),
            String::from("  #######        S   P    #####.#  "),
            String::from("  #.#...#                 #......VT"),
            String::from("  #.#.#.#                 #.#####  "),
            String::from("  #...#.#               YN....#.#  "),
            String::from("  #.###.#                 #####.#  "),
            String::from("DI....#.#                 #.....#  "),
            String::from("  #####.#                 #.###.#  "),
            String::from("ZZ......#               QG....#..AS"),
            String::from("  ###.###                 #######  "),
            String::from("JO..#.#.#                 #.....#  "),
            String::from("  #.#.#.#                 ###.#.#  "),
            String::from("  #...#..DI             BU....#..LF"),
            String::from("  #####.#                 #.#####  "),
            String::from("YN......#               VT..#....QG"),
            String::from("  #.###.#                 #.###.#  "),
            String::from("  #.#...#                 #.....#  "),
            String::from("  ###.###    J L     J    #.#.###  "),
            String::from("  #.....#    O F     P    #.#...#  "),
            String::from("  #.###.#####.#.#####.#####.###.#  "),
            String::from("  #...#.#.#...#.....#.....#.#...#  "),
            String::from("  #.#####.###.###.#.#.#########.#  "),
            String::from("  #...#.#.....#...#.#.#.#.....#.#  "),
            String::from("  #.###.#####.###.###.#.#.#######  "),
            String::from("  #.#.........#...#.............#  "),
            String::from("  #########.###.###.#############  "),
            String::from("           B   J   C               "),
            String::from("           U   P   P               "),
        ]);

        let len = map.find_path_len();
        assert_eq!(len, 58);
    }
}
