use std::fs::File;
use std::io::{BufRead, BufReader};

struct Edge {
    p1: (i64, i64),
    p2: (i64, i64),
}

impl Edge {
    fn is_horizontal(&self) -> bool {
        return self.p1.1 == self.p2.1;
    }

    fn len(&self) -> i64 {
        return (self.p2.0 - self.p1.0).abs() + (self.p2.1 - self.p1.1).abs();
    }

    // Distance from start of the edge to a given point
    fn distance_along(&self, p: (i64, i64)) -> i64 {
        return (p.0 - self.p1.0).abs() + (p.1 - self.p1.1).abs();
    }
}

// Find the intersection between two (axis-aligned) edges.
fn find_intersection(e1: &Edge, e2: &Edge) -> Option<(i64, i64)> {
    if e1.is_horizontal() {
        if e2.is_horizontal() {
            return None;
        }

        let e1_y = e1.p1.1;
        let e2_x = e2.p1.0;

        if ((e2.p1.1 - e1_y).signum() != (e2.p2.1 - e1_y).signum()) &&
           ((e1.p1.0 - e2_x).signum() != (e1.p2.0 - e2_x).signum()) {
            return Some((e2_x, e1_y));
        }
    }

    if e2.is_horizontal() {
        if e1.is_horizontal() {
            return None;
        }

        let e1_x = e1.p1.0;
        let e2_y = e2.p1.1;

        if ((e1.p1.1 - e2_y).signum() != (e1.p2.1 - e2_y).signum()) &&
           ((e2.p1.0 - e1_x).signum() != (e2.p2.0 - e1_x).signum()) {
            return Some((e1_x, e2_y));
        }
    }

    return None;
}

// Return the sum of the distances along both wires for each intersection on two wires.
fn find_intersections(w1: &Vec<Edge>, w2: &Vec<Edge>) -> Vec<i64> {
    let mut intersections = Vec::new();
    let mut w1_dist = 0;
    for e1 in w1 {
        let mut w2_dist = 0;
        for e2 in w2 {
            match find_intersection(e1, e2) {
                Some(i) => {
                    // Find the distance along the two wires - i.e. the distance along all completed
                    // edges so far, plus the partial distance along the intersecting edges.
                    let dist = w1_dist + e1.distance_along(i) + w2_dist + e2.distance_along(i);
                    intersections.push(dist);
                },
                None => ()
            };

            w2_dist += e2.len();
        }

        w1_dist += e1.len();
    }

    return intersections;
}

fn parse_wire(edges: &[String]) -> Vec<Edge> {
    let mut graph = Vec::new();
    let mut current_pos = (0, 0);
    for e in edges {
        let (dir, dist) = e.split_at(1);
        let dist = dist.parse::<i64>().expect("Failed to parse value");

        let end = match dir {
            "U" => (current_pos.0, current_pos.1 + dist),
            "D" => (current_pos.0, current_pos.1 - dist),
            "R" => (current_pos.0 + dist, current_pos.1),
            "L" => (current_pos.0 - dist, current_pos.1),
            _ => current_pos,
        };

        let new_edge = Edge{p1: current_pos, p2: end};
        graph.push(new_edge);

        current_pos = end;
    }

    return graph;
}

fn read_wires() -> Vec<Vec<Edge>> {
    let file = File::open("input").expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut wires = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let edges: Vec<String> = line.trim().split(",").map(|s| String::from(s)).collect();
        let wire = parse_wire(&edges);
        wires.push(wire);
    }

    return wires;
}

fn main() {
    let wires = read_wires();
    let wire_a = &wires[0];
    let wire_b = &wires[1];

    let intersections: Vec<i64> = find_intersections(wire_a, wire_b);
    let result = intersections
        .into_iter()
        .filter(|d| *d > 0)
        .min()
        .expect("No intersections");

    println!("Result: {}", result);
}
