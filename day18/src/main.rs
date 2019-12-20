use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Coords = (usize, usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum Tile {
    Wall,
    Floor,
    Entrance(Coords),
    Key(char),
    Door(char),
}

impl Tile {
    fn from_char(c: char, coords: Coords) -> Self {
        match c {
            '#' => Tile::Wall,
            '.' => Tile::Floor,
            '@' => Tile::Entrance(coords),
            'a'..='z' => Tile::Key(c),
            'A'..='Z' => Tile::Door(c.to_lowercase().next().unwrap()),
            _ => panic!("Unexpected tile char"),
        }
    }
}

#[derive(Debug)]
struct Map {
    tiles: Vec<Vec<Tile>>,
    starts: Vec<Tile>,
    keys: HashMap<char, Coords>,

    // Map from a tile (the entrance or a key) to a vector containing
    // the keys that can be reached from that key, the distance for
    // each key, and any doors that need to be unlocked.
    reachability: HashMap<Tile, Vec<(char, usize, HashSet<char>)>>,
}

impl Map {
    fn from_lines(lines: &[String]) -> Self {
        let mut tiles = Vec::new();
        let mut starts = Vec::new();
        let mut keys = HashMap::new();
        for (y, line) in lines.iter().enumerate() {
            let mut row = Vec::new();
            for (x, c) in line.chars().enumerate() {
                let coords = (x, y);
                let tile = Tile::from_char(c, coords);
                row.push(tile);
                match tile {
                    Tile::Entrance(_) => starts.push(tile),
                    Tile::Key(c) => {
                        let _ = keys.insert(c, coords);
                    }
                    _ => (),
                }
            }
            tiles.push(row);
        }

        Map {
            tiles: tiles,
            starts: starts,
            keys: keys,
            reachability: HashMap::new(),
        }
    }

    fn from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        Map::from_lines(&lines)
    }

    fn get_neighbouring_tiles(&self, coords: Coords) -> Vec<Coords> {
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

    fn get_tile(&self, coords: Coords) -> Tile {
        self.tiles[coords.1][coords.0]
    }

    fn find_keys_from_coords(self: &Self, coords: Coords) -> Vec<(char, usize, HashSet<char>)> {
        let mut keys: HashMap<char, (usize, HashSet<char>)> = HashMap::new();

        let mut visited: HashSet<Coords> = HashSet::new();
        let mut queue: VecDeque<(Coords, usize, HashSet<char>)> = VecDeque::new();
        queue.push_back((coords, 0, HashSet::new()));

        while !queue.is_empty() {
            let (coords, d, required_keys) = queue.pop_front().unwrap();

            visited.insert(coords);
            let distance = d + 1;

            for coords in self.get_neighbouring_tiles(coords) {
                if visited.contains(&coords) {
                    continue;
                }

                match self.get_tile(coords) {
                    Tile::Key(c) => {
                        keys.insert(c, (distance, required_keys.clone()));
                        queue.push_back((coords, distance, required_keys.clone()));
                    }
                    Tile::Door(c) => {
                        let mut new_required_keys = required_keys.clone();
                        new_required_keys.insert(c);
                        queue.push_back((coords, distance, new_required_keys));
                    }
                    Tile::Floor | Tile::Entrance(_) => {
                        queue.push_back((coords, distance, required_keys.clone()))
                    }

                    _ => (),
                }
            }
        }

        keys.into_iter().map(|(k, (d, ks))| (k, d, ks)).collect()
    }

    fn build_reachability(&mut self) {
        let start_info: HashMap<Tile, _> = self
            .starts
            .iter()
            .map(|tile| match tile {
                Tile::Entrance(coords) => (*tile, self.find_keys_from_coords(*coords)),
                _ => panic!("Wrong start tile type"),
            })
            .collect();
        let key_info: HashMap<Tile, _> = self
            .keys
            .iter()
            .map(|(k, coords)| (Tile::Key(*k), self.find_keys_from_coords(*coords)))
            .collect();

        self.reachability.extend(start_info);
        self.reachability.extend(key_info);
    }

    fn make_memo_key(current_locs: &Vec<Tile>, keys: &HashSet<char>) -> String {
        // Don't sort the locations - the order is important for the case where more
        // than one current location is at an entrance
        let loc_str: String = current_locs
            .iter()
            .map(|t| match t {
                Tile::Entrance(_) => '@',
                Tile::Key(c) => *c,
                _ => panic!("Current location neither an entrance nor a key"),
            })
            .collect();

        let mut keyvec = Vec::new();
        for c in keys {
            keyvec.push(*c);
        }
        keyvec.sort();

        format!("{}{}", loc_str, keyvec.iter().collect::<String>())
    }

    fn find_shortest_path(
        &self,
        keys: HashSet<char>,
        current_tiles: Vec<Tile>,
        memo: &mut HashMap<String, usize>,
    ) -> usize {
        if keys.len() == self.keys.len() {
            return 0;
        }

        let mut all_distances = Vec::new();
        for i in 0..current_tiles.len() {
            let distances: Vec<usize> = self.reachability[&current_tiles[i]]
                .iter()
                .filter(|(c, _, req_keys)| !keys.contains(c) && req_keys.is_subset(&keys))
                .map(|(c, d, _)| {
                    let mut new_current_tiles = current_tiles.clone();
                    new_current_tiles[i] = Tile::Key(*c);

                    let memo_key = Map::make_memo_key(&new_current_tiles, &keys);
                    if let Some(distance) = memo.get(&memo_key) {
                        d + *distance
                    } else {
                        let mut new_keys = keys.clone();
                        new_keys.insert(*c);

                        let distance = self.find_shortest_path(new_keys, new_current_tiles, memo);
                        memo.insert(memo_key, distance);
                        d + distance
                    }
                })
                .collect();

            if !distances.is_empty() {
                all_distances.push(*distances.iter().min().unwrap());
            }
        }

        if all_distances.is_empty() {
            return 0;
        } else {
            return *all_distances.iter().min().unwrap();
        }
    }
}

fn main() {
    let mut map = Map::from_file("input");
    map.build_reachability();
    let shortest = map.find_shortest_path(HashSet::new(), map.starts.clone(), &mut HashMap::new());
    println!("Part 1: {}", shortest);

    let mut map = Map::from_file("input2");
    map.build_reachability();
    let shortest = map.find_shortest_path(HashSet::new(), map.starts.clone(), &mut HashMap::new());
    println!("Part 2: {}", shortest);
}
