use regex::Regex;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Copy, Clone, Debug)]
enum ShuffleType {
    Stack,
    Cut(i64),
    Increment(i64),
}

fn inverse_mod(a: i64, n: i64) -> i64 {
    let mut t = 0;
    let mut r = n;
    let mut newt = 1;
    let mut newr = a;
    while newr != 0 {
        let quot = r / newr;

        let oldt = t;
        t = newt;
        newt = oldt - quot * newt;

        let oldr = r;
        r = newr;
        newr = oldr - quot * newr;
    }

    if r > 1 {
        panic!("invalid inverse mod");
    }

    if t < 0 {
        t = t + n;
    }

    t
}

fn r#mod(a: i64, m: i64) -> i64 {
    // Rust's % operator is remainder rather than modulus,
    // so need to adjust for negative numbers.
    if a < 0 {
        ((a % m) + m) % m
    } else {
        a % m
    }
}

fn shuffle(num_cards: i64, shuffles: &VecDeque<ShuffleType>, mut index: i64) -> i64 {
    let mut shuffle: VecDeque<ShuffleType> = shuffles.clone();

    while !shuffle.is_empty() {
        let op = shuffle.pop_front().unwrap();

        match op {
            ShuffleType::Stack => index = (num_cards - 1) - index,
            ShuffleType::Cut(cut) => index = r#mod(index + (num_cards - cut), num_cards),
            ShuffleType::Increment(inc) => index = (index * inc) % num_cards,
        }
    }

    index
}

fn reverse_shuffle(num_cards: i64, shuffles: &VecDeque<ShuffleType>, mut index: i64) -> i64 {
    let mut shuffle: VecDeque<ShuffleType> = shuffles.clone();

    while !shuffle.is_empty() {
        let op = shuffle.pop_back().unwrap();

        // Reverse the operation.
        match op {
            // Forward is index' = (num_cards - 1) - index
            // Reverse is index = (num_cards - 1) - index'
            ShuffleType::Stack => index = (num_cards - 1) - index,
            // Forward is index' = (index + (num_cards - cut)) % num_cards
            // Reverse is index = (index' - (num_cards - cut)) % num_cards ?
            ShuffleType::Cut(cut) => index = r#mod(index - (num_cards - cut), num_cards),
            // Forward is index' = (index * inc) % num_cards
            // Reverse is complicated :)
            ShuffleType::Increment(inc) => index = (index * inverse_mod(inc, num_cards)) % num_cards,
        }
    }

    index
}

fn parse_input(filename: &str) -> VecDeque<ShuffleType> {
    let stack_re = Regex::new(r"deal into new stack").unwrap();
    let cut_re = Regex::new(r"cut (?P<cut>-?\d+)").unwrap();
    let inc_re = Regex::new(r"deal with increment (?P<inc>\d+)").unwrap();

    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut shuffles = VecDeque::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();

        if stack_re.captures(line).is_some() {
            shuffles.push_back(ShuffleType::Stack);
        } else if let Some(caps) = cut_re.captures(line) {
            let cut = caps["cut"].parse::<i64>().expect("Malformed cut size");
            shuffles.push_back(ShuffleType::Cut(cut));
        } else if let Some(caps) = inc_re.captures(line) {
            let inc = caps["inc"].parse::<i64>().expect("Malformed increment");
            shuffles.push_back(ShuffleType::Increment(inc));
        } else {
            panic!("Unexpected shuffle");
        }
    }

    shuffles
}

fn main() {
    // Part 1
    const PT1_NUM_CARDS: i64 = 10007;
    const PT1_TGT_INDEX: i64 = 2019;

    let shuffles = parse_input("input");
    let result = shuffle(PT1_NUM_CARDS, &shuffles, PT1_TGT_INDEX);
    println!("Card {} at index: {}", PT1_TGT_INDEX, result);

    // Part 2
    // TODO: This is too slow, need to combine the operations into 1
    const PT2_SHUFFLE_COUNT: i64 = 101741582076661;
    const PT2_NUM_CARDS: i64 = 119315717514047;
    const PT2_TGT_INDEX: i64 = 2020;

    let mut index = PT2_TGT_INDEX;
    for i in 0..PT2_SHUFFLE_COUNT {
        println!("{} {}", i, index);
        index = reverse_shuffle(PT2_NUM_CARDS, &shuffles, index);
    }
    println!("Card at index {}: {}", PT2_TGT_INDEX, index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_reverse() {
        let shuffles = VecDeque::from(vec![ShuffleType::Stack]);
        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 8);

        let result = reverse_shuffle(10, &shuffles, 7);
        assert_eq!(result, 2);
    }

    #[test]
    fn cut_reverse() {
        let shuffles = VecDeque::from(vec![ShuffleType::Cut(3)]);
        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 4);

        let result = reverse_shuffle(10, &shuffles, 9);
        assert_eq!(result, 2);
    }

    #[test]
    fn cut_negative_reverse() {
        let shuffles = VecDeque::from(vec![ShuffleType::Cut(-4)]);

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 7);

        let result = reverse_shuffle(10, &shuffles, 9);
        assert_eq!(result, 5);
    }

    #[test]
    fn increment_reverse() {
        let shuffles = VecDeque::from(vec![ShuffleType::Increment(3)]);
        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 7);

        let result = reverse_shuffle(10, &shuffles, 9);
        assert_eq!(result, 3);
    }

    #[test]
    fn ex1_reverse() {
        let shuffles = VecDeque::from(vec![
            ShuffleType::Increment(7),
            ShuffleType::Stack,
            ShuffleType::Stack,
        ]);

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 3);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 8);
    }

    #[test]
    fn ex2_reverse() {
        let shuffles = VecDeque::from(vec![
            ShuffleType::Cut(6),
            ShuffleType::Increment(7),
            ShuffleType::Stack,
        ]);

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 0);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 5);
    }

    #[test]
    fn ex3_reverse() {
        let shuffles = VecDeque::from(vec![
            ShuffleType::Increment(7),
            ShuffleType::Increment(9),
            ShuffleType::Cut(-2),
        ]);

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 3);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 8);
    }

    #[test]
    fn ex4_reverse() {
        let shuffles = VecDeque::from(vec![
            ShuffleType::Stack,
            ShuffleType::Cut(-2),
            ShuffleType::Increment(7),
            ShuffleType::Cut(8),
            ShuffleType::Cut(-4),
            ShuffleType::Increment(7),
            ShuffleType::Cut(3),
            ShuffleType::Increment(9),
            ShuffleType::Increment(3),
            ShuffleType::Cut(-1),
        ]);

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 2);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 7);
    }
}
