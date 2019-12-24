use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

const MAP_SIZE: usize = 5;

#[derive(Copy, Clone, Debug)]
enum Tile {
    Empty,
    Bug,
}

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '#' => Tile::Bug,
            '.' => Tile::Empty,
            _ => panic!("Unknown tile type"),
        }
    }

    fn to_char(&self) -> char {
        match self {
            Tile::Bug => '#',
            Tile::Empty => '.',
        }
    }

    fn is_bug(&self) -> bool {
        match self {
            Tile::Bug => true,
            Tile::Empty => false,
        }
    }
}

type Coords = (usize, usize);

#[derive(Clone)]
struct Map {
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    fn empty() -> Self {
        Map {
            tiles: vec![vec![Tile::Empty; MAP_SIZE]; MAP_SIZE],
        }
    }

    fn from_lines(lines: &Vec<String>) -> Self {
        let mut tiles = Vec::new();

        for l in lines {
            let row = l.chars().map(|c| Tile::from_char(c)).collect();
            tiles.push(row);
        }

        Map { tiles: tiles }
    }

    fn from_file(filename: &str) -> Self {
        let file = File::open(filename).expect("Failed to open file");
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            let line = String::from(line.trim());
            lines.push(line);
        }

        Self::from_lines(&lines)
    }

    fn to_hash(&self) -> String {
        self.tiles
            .iter()
            .map(|row| row.iter().map(|t| t.to_char()).collect::<String>())
            .collect::<Vec<String>>()
            .join("")
    }

    fn biodiversity(&self) -> u64 {
        self.tiles
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, t)| t.is_bug())
            .fold(0, |acc, (i, _)| acc + 2u64.pow(i as u32))
    }

    fn get_neighbour_coords_for_inner(&self, inner_coords: Coords) -> Vec<Coords> {
        let mut neighbours = Vec::new();

        // Hardcoding the tile coords, meh :)
        if inner_coords.0 == 0 {
            neighbours.push((1, 2));
        }
        if inner_coords.1 == 0 {
            neighbours.push((2, 1));
        }
        if inner_coords.0 == 4 {
            neighbours.push((3, 2));
        }
        if inner_coords.1 == 4 {
            neighbours.push((2, 3));
        }

        neighbours
    }

    // self is the "outer" map.
    fn get_neighbour_bug_count_for_inner(&self, inner_coords: Coords) -> usize {
        self.get_neighbour_coords_for_inner(inner_coords)
            .iter()
            .filter(|(x, y)| self.tiles[*y][*x].is_bug())
            .count()
    }

    fn get_neighbour_coords_for_outer(&self, outer_coords: Coords) -> Vec<Coords> {
        // Again hardcoding the tile coords.
        if outer_coords == (1, 2) {
            // Left inner tile, get all of the left hand side of this map.
            return vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)];
        }
        if outer_coords == (3, 2) {
            // Right inner tile, get all the right hand side of this map.
            return vec![(4, 0), (4, 1), (4, 2), (4, 3), (4, 4)];
        }
        if outer_coords == (2, 1) {
            // Top inner tile, get all the top side of this map.
            return vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)];
        }
        if outer_coords == (2, 3) {
            // Bottom inner tile, get all the bottom side of this map.
            return vec![(0, 4), (1, 4), (2, 4), (3, 4), (4, 4)];
        }

        return vec![];
    }

    // self is the "inner" map.
    fn get_neighbour_bug_count_for_outer(&self, outer_coords: Coords) -> usize {
        self.get_neighbour_coords_for_outer(outer_coords)
            .iter()
            .filter(|(x, y)| self.tiles[*y][*x].is_bug())
            .count()
    }

    fn get_neighbour_coords(&self, coords: Coords) -> Vec<Coords> {
        let mut neighbours = Vec::new();

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
    }

    fn get_neighbour_bug_count(&self, coords: Coords) -> usize {
        self.get_neighbour_coords(coords)
            .iter()
            .filter(|(x, y)| self.tiles[*y][*x].is_bug())
            .count()
    }

    fn evolve_tile(&self, tile: Tile, bug_count: usize) -> Tile {
        match tile {
            Tile::Bug => {
                if bug_count == 1 {
                    Tile::Bug
                } else {
                    Tile::Empty
                }
            }
            Tile::Empty => {
                if bug_count == 1 || bug_count == 2 {
                    Tile::Bug
                } else {
                    Tile::Empty
                }
            }
        }
    }

    fn evolve(&mut self) {
        self.evolve_infinite(None, None);
    }

    fn evolve_infinite(&mut self, inner: Option<&Map>, outer: Option<&Map>) {
        let mut new_tiles = Vec::new();
        for y in 0..self.tiles.len() {
            let old_row = &self.tiles[y];
            let mut new_row = Vec::new();

            for x in 0..old_row.len() {
                let coords = (x, y);

                if inner.is_some() && coords == (2, 2) {
                    // If we have an inner map, the middle tile stays empty.
                    new_row.push(Tile::Empty);
                    continue;
                }

                let bug_count = self.get_neighbour_bug_count(coords)
                    + inner.map_or(0, |i| i.get_neighbour_bug_count_for_outer(coords))
                    + outer.map_or(0, |o| o.get_neighbour_bug_count_for_inner(coords));

                new_row.push(self.evolve_tile(self.tiles[y][x], bug_count));
            }

            new_tiles.push(new_row);
        }

        self.tiles = new_tiles;
    }

    fn evolve_til_stable(&mut self) {
        let mut evolutions = HashSet::new();
        evolutions.insert(self.to_hash());

        loop {
            self.evolve();
            let hash = self.to_hash();
            if evolutions.contains(&hash) {
                break;
            }
            evolutions.insert(hash);
        }
    }

    fn count_bugs(&self) -> usize {
        self.tiles.iter().fold(0, |acc, row| {
            acc + row.iter().filter(|t| t.is_bug()).count()
        })
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for row in &self.tiles {
            let row_str: String = row.iter().map(|t| t.to_char()).collect();
            output = format!("{}\n{}", output, row_str);
        }
        write!(f, "{}", output)
    }
}

#[derive(Debug)]
struct InfiniteMap {
    levels: VecDeque<Map>,
}

impl InfiniteMap {
    fn from_lines(lines: &Vec<String>) -> Self {
        InfiniteMap {
            levels: VecDeque::from(vec![Map::from_lines(lines)]),
        }
    }
    fn from_file(filename: &str) -> Self {
        InfiniteMap {
            levels: VecDeque::from(vec![Map::from_file(filename)]),
        }
    }

    fn evolve(&mut self) {
        self.levels.push_front(Map::empty());
        self.levels.push_back(Map::empty());

        let mut new_levels = VecDeque::new();

        for i in 0..self.levels.len() {
            let inner = if i > 0 {
                Some(&self.levels[i - 1])
            } else {
                None
            };

            let outer = if i < self.levels.len() - 1 {
                Some(&self.levels[i + 1])
            } else {
                None
            };

            let mut new_level = self.levels[i].clone();
            new_level.evolve_infinite(inner, outer);
            new_levels.push_back(new_level);
        }

        self.levels = new_levels;
    }

    fn count_bugs(&self) -> usize {
        self.levels
            .iter()
            .fold(0, |acc, map| acc + map.count_bugs())
    }
}

fn main() {
    // Part 1
    let mut map = Map::from_file("input");
    map.evolve_til_stable();
    println!("Part 1: Biodiversity {}", map.biodiversity());

    // Part 2
    let mut inf_map = InfiniteMap::from_file("input");
    const EVOLUTIONS: isize = 200;
    for _ in 0..EVOLUTIONS {
        inf_map.evolve();
    }
    println!("Part 2: Bugs {}", inf_map.count_bugs());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1() {
        let mut map = Map::from_lines(&vec![
            String::from("....#"),
            String::from("#..#."),
            String::from("#..##"),
            String::from("..#.."),
            String::from("#...."),
        ]);

        map.evolve_til_stable();
        assert_eq!(map.biodiversity(), 2129920);
    }

    #[test]
    fn part2() {
        let mut inf_map = InfiniteMap::from_lines(&vec![
            String::from("....#"),
            String::from("#..#."),
            String::from("#..##"),
            String::from("..#.."),
            String::from("#...."),
        ]);
        for _ in 0..10 {
            inf_map.evolve();
        }
        assert_eq!(inf_map.count_bugs(), 99);
    }
}
