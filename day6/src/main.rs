use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Body {
    label: String,
    satellites: Vec<Body>,
}

fn build_tree(label: &String, edges: &HashMap<String, Vec<String>>) -> Body {
    let mut satellites = Vec::new();
    if let Some(sat_labels) = edges.get(label) {
        for sat_label in sat_labels {
            satellites.push(build_tree(sat_label, edges));
        }
    }

    return Body {
        label: label.clone(),
        satellites: satellites,
    };
}

// Build a tree of orbits from the input file.
fn parse_input(filename: &str) -> Body {
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    let re = Regex::new(r"(?P<inner>.*)\)(?P<outer>.*)").unwrap();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();
        let caps = re.captures(line).expect("Malformed line");
        let inner = String::from(&caps["inner"]);
        let outer = String::from(&caps["outer"]);

        if let Some(nodes) = edges.get_mut(&inner) {
            nodes.push(outer);
        } else {
            edges.insert(inner, vec![outer]);
        }
    }

    let root_label = String::from("COM");
    return build_tree(&root_label, &edges);
}

// The minimal orbital transfer distance between us and santa is
// found by finding the lowest common ancestor of those two nodes
// in the tree of orbits, and summing the distance between the
// us/santa nodes and the LCA.
fn find_lca_distance(tree: &Body, depth: u32) -> Option<(u32)> {
    match tree.label.as_ref() {
        "SAN" => return Some(depth),
        "YOU" => return Some(depth),
        _ => {
            let results: Vec<u32> = tree
                .satellites
                .iter()
                .filter_map(|s| find_lca_distance(&s, depth + 1))
                .collect();

            return match results.len() {
                // 2 matches: child branches have both us and santa, this is
                // the LCA. Return the distance between the two.
                2 => {
                    let sum: u32 = results.iter().sum();
                    Some(sum - depth * 2 - 2)
                }
                // 1 match, either one of the child branches has either us or
                // santa, or we already found the LCA. Just return the result.`
                1 => {
                    let val: u32 = *results.first().unwrap();
                    Some(val)
                }
                0 => None,
                _ => panic!("Found more than 2 branch matches"),
            };
        }
    }
}

fn main() {
    let com = parse_input("input");
    let distance = find_lca_distance(&com, 0).expect("Couldn't find distance");
    println!("Distance: {}", distance);
}
