use pathfinding::prelude::dijkstra;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Coords2D = (usize, usize);
type Coords3D = (usize, usize, usize);

#[derive(Debug)]
enum Tile {
    Empty,
    Floor,
    Wall,
    Warp(Coords2D),
}

#[derive(Debug)]
struct Map {
    tiles: Vec<Vec<Tile>>,
    warps: Vec<Coords2D>,
    start: Coords3D,
    end: Coords3D,
}

#[derive(Copy, Clone)]
enum Part {
    One,
    Two,
}

impl Map {
    fn find_tile_labels(coords: Coords2D, lines: &Vec<Vec<char>>) -> Option<String> {
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
        let mut warps: HashMap<String, Coords2D> = HashMap::new();
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
                                    start = Some((coords.0, coords.1, 0));
                                    Tile::Floor
                                }
                                "ZZ" => {
                                    end = Some((coords.0, coords.1, 0));
                                    Tile::Floor
                                }
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
                                }
                            }
                        } else {
                            Tile::Floor
                        }
                    }
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

    fn get_warp_location(&self, current_coords: Coords3D, warp_coords: Coords2D, part: Part) -> Option<Coords3D> {
        match part {
            Part::One => Some((warp_coords.0, warp_coords.1, current_coords.2)),
            Part::Two => {
                if current_coords.0 == 0 || current_coords.1 == 0 ||
                    current_coords.0 == self.tiles[0].len() - 1 ||
                    current_coords.1 == self.tiles.len() - 1 {
                    // This is on the outer edge of the map, go up a level
                    // if we can (ie. not at the top level)
                    if current_coords.2 > 0 {
                        Some((warp_coords.0, warp_coords.1, current_coords.2 - 1))
                    } else {
                        None
                    }
                } else {
                    Some((warp_coords.0, warp_coords.1, current_coords.2 + 1))
                }
            },
        }
    }

    fn get_neighbours(&self, coords: Coords3D, part: Part) -> Vec<Coords3D> {
        let mut neighbours: Vec<Coords3D> = Vec::new();

        // If this is a warp tile, add the other end as a neighbour.
        match self.tiles[coords.1][coords.0] {
            Tile::Warp(c) => {
                let nbr = self.get_warp_location(coords, c, part);
                if nbr.is_some() {
                    neighbours.push(nbr.unwrap());
                }
            },
            _ => (),
        }

        // Add "normal" neighbours.
        if coords.0 > 0 {
            neighbours.push((coords.0 - 1, coords.1, coords.2));
        }
        if coords.0 < self.tiles[0].len() - 1 {
            neighbours.push((coords.0 + 1, coords.1, coords.2));
        }
        if coords.1 > 0 {
            neighbours.push((coords.0, coords.1 - 1, coords.2));
        }
        if coords.1 < self.tiles.len() - 1 {
            neighbours.push((coords.0, coords.1 + 1, coords.2));
        }

        neighbours
            .iter()
            .cloned()
            .filter(|(x, y, _)| match self.tiles[*y][*x] {
                Tile::Floor => true,
                Tile::Warp(_) => true,
                _ => false,
            })
            .collect()
    }

    fn find_path_len(&self, part: Part) -> usize {
        let successors = |&coords: &Coords3D| -> Vec<(Coords3D, usize)> {
            self.get_neighbours(coords, part)
                .into_iter()
                .map(|coords| (coords, 1))
                .collect()
        };

        let path = dijkstra(&self.start, successors, |&coords| coords == self.end);
        path.map(|tup| tup.1).unwrap()
    }
}

fn main() {
    let map = Map::from_file("input");
    let len = map.find_path_len(Part::One);
    println!("Shortest Path for part 1: {:?}", len);

    let len = map.find_path_len(Part::Two);
    println!("Shortest Path for part 2: {:?}", len);
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

        let len = map.find_path_len(Part::One);
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

        let len = map.find_path_len(Part::One);
        assert_eq!(len, 58);
    }

    #[test]
    fn pt2_ex2() {
        let map = Map::from_lines(&vec![
            String::from("             Z L X W       C                 "),
            String::from("             Z P Q B       K                 "),
            String::from("  ###########.#.#.#.#######.###############  "),
            String::from("  #...#.......#.#.......#.#.......#.#.#...#  "),
            String::from("  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###  "),
            String::from("  #.#...#.#.#...#.#.#...#...#...#.#.......#  "),
            String::from("  #.###.#######.###.###.#.###.###.#.#######  "),
            String::from("  #...#.......#.#...#...#.............#...#  "),
            String::from("  #.#########.#######.#.#######.#######.###  "),
            String::from("  #...#.#    F       R I       Z    #.#.#.#  "),
            String::from("  #.###.#    D       E C       H    #.#.#.#  "),
            String::from("  #.#...#                           #...#.#  "),
            String::from("  #.###.#                           #.###.#  "),
            String::from("  #.#....OA                       WB..#.#..ZH"),
            String::from("  #.###.#                           #.#.#.#  "),
            String::from("CJ......#                           #.....#  "),
            String::from("  #######                           #######  "),
            String::from("  #.#....CK                         #......IC"),
            String::from("  #.###.#                           #.###.#  "),
            String::from("  #.....#                           #...#.#  "),
            String::from("  ###.###                           #.#.#.#  "),
            String::from("XF....#.#                         RF..#.#.#  "),
            String::from("  #####.#                           #######  "),
            String::from("  #......CJ                       NM..#...#  "),
            String::from("  ###.#.#                           #.###.#  "),
            String::from("RE....#.#                           #......RF"),
            String::from("  ###.###        X   X       L      #.#.#.#  "),
            String::from("  #.....#        F   Q       P      #.#.#.#  "),
            String::from("  ###.###########.###.#######.#########.###  "),
            String::from("  #.....#...#.....#.......#...#.....#.#...#  "),
            String::from("  #####.#.###.#######.#######.###.###.#.#.#  "),
            String::from("  #.......#.......#.#.#.#.#...#...#...#.#.#  "),
            String::from("  #####.###.#####.#.#.#.#.###.###.#.###.###  "),
            String::from("  #.......#.....#.#...#...............#...#  "),
            String::from("  #############.#.#.###.###################  "),
            String::from("               A O F   N                     "),
            String::from("               A A D   M                     "),
        ]);

        let len = map.find_path_len(Part::Two);
        assert_eq!(len, 396);
    }
}
