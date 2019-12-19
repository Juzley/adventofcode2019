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
    Entrance,
    Key(char),
    Door(char),
}

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '#' => Tile::Wall,
            '.' => Tile::Floor,
            '@' => Tile::Entrance,
            'a'..='z' => Tile::Key(c),
            'A'..='Z' => Tile::Door(c.to_lowercase().next().unwrap()),
            _ => panic!("Unexpected tile char"),
        }
    }
}

#[derive(Debug)]
struct Map {
    tiles: Vec<Vec<Tile>>,
    start: Coords,
    keys: HashMap<char, Coords>,

    // Map from a tile (the entrance or a key) to a vector containing
    // the keys that can be reached from that key, the distance for
    // each key, and any doors that need to be unlocked.
    reachability: HashMap<Tile, Vec<(char, usize, HashSet<char>)>>,
}

impl Map {
    fn from_lines(lines: &[String]) -> Self {
        let tiles: Vec<Vec<Tile>> = lines
            .iter()
            .map(|l| l.chars().map(|c| Tile::from_char(c)).collect::<Vec<Tile>>())
            .collect();

        let mut start = None;
        let mut keys = HashMap::new();
        for (y, row) in tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let coords = (x, y);
                match tile {
                    Tile::Entrance => start = Some(coords),
                    Tile::Key(c) => {
                        let _ = keys.insert(*c, coords);
                    }
                    _ => (),
                }
            }
        }

        Map {
            tiles: tiles,
            start: start.unwrap(),
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
                    Tile::Floor | Tile::Entrance => {
                        queue.push_back((coords, distance, required_keys.clone()))
                    }

                    _ => (),
                }
            }
        }

        keys.into_iter().map(|(k, (d, ks))| (k, d, ks)).collect()
    }

    fn build_reachability(&mut self) {
        let start_info = self.find_keys_from_coords(self.start);
        let key_info: HashMap<Tile, _> = self
            .keys
            .iter()
            .map(|(k, coords)| (Tile::Key(*k), self.find_keys_from_coords(*coords)))
            .collect();

        self.reachability.insert(Tile::Entrance, start_info);
        self.reachability.extend(key_info);
    }

    fn make_memo_key(key: char, keys: &HashSet<char>) -> String {
        let mut keyvec = Vec::new();
        for c in keys {
            keyvec.push(*c);
        }
        keyvec.sort();

        format!("{}{}", key.to_string(), keyvec.iter().collect::<String>())
    }

    fn find_shortest_path(
        &self,
        keys: HashSet<char>,
        tile: Tile,
        memo: &mut HashMap<String, usize>,
    ) -> usize {
        let reachability: Vec<usize> = self.reachability[&tile]
            .iter()
            .filter(|(c, _, req_keys)| !keys.contains(c) && req_keys.is_subset(&keys))
            .map(|(c, d, _)| {
                let memo_key = Map::make_memo_key(*c, &keys);
                if let Some(distance) = memo.get(&memo_key) {
                    d + *distance
                } else {
                    let mut keys = keys.clone();
                    keys.insert(*c);
                    let distance = self.find_shortest_path(keys, Tile::Key(*c), memo);
                    let _ = memo.insert(memo_key, distance);
                    d + distance
                }
            })
            .collect();

        if reachability.is_empty() {
            return 0;
        } else {
            return *reachability.iter().min().unwrap();
        }
    }
}

fn main() {
    let mut map = Map::from_file("input");
    map.build_reachability();
    println!("{:?}", map);

    let shortest = map.find_shortest_path(HashSet::new(), Tile::Entrance, &mut HashMap::new());
    println!("{}", shortest);
}
