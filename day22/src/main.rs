use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Copy, Clone, Debug)]
enum ShuffleType {
    Stack,
    Cut(i128),
    Increment(i128),
}

impl ShuffleType {
    fn inverse(&self, num_cards: i128) -> Self {
        match self {
            &ShuffleType::Stack => ShuffleType::Stack,
            &ShuffleType::Cut(n) => ShuffleType::Cut(-n),
            &ShuffleType::Increment(n) => {
                ShuffleType::Increment(inverse_mod(n, num_cards) % num_cards)
            }
        }
    }

    fn to_multiply_add(&self, num_cards: i128) -> (i128, i128) {
        match self {
            &ShuffleType::Stack => (num_cards - 1, num_cards - 1),
            &ShuffleType::Cut(n) => (1, (num_cards - n) % num_cards),
            &ShuffleType::Increment(n) => (n % num_cards, 0),
        }
    }
}

fn inverse_mod(a: i128, n: i128) -> i128 {
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

fn r#mod(a: i128, m: i128) -> i128 {
    // Rust's % operator is remainder rather than modulus,
    // so need to adjust for negative numbers.
    if a < 0 {
        ((a % m) + m) % m
    } else {
        a % m
    }
}

fn parse_input(filename: &str) -> Vec<ShuffleType> {
    let stack_re = Regex::new(r"deal into new stack").unwrap();
    let cut_re = Regex::new(r"cut (?P<cut>-?\d+)").unwrap();
    let inc_re = Regex::new(r"deal with increment (?P<inc>\d+)").unwrap();

    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut shuffles = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();

        if stack_re.captures(line).is_some() {
            shuffles.push(ShuffleType::Stack);
        } else if let Some(caps) = cut_re.captures(line) {
            let cut = caps["cut"].parse::<i128>().expect("Malformed cut size");
            shuffles.push(ShuffleType::Cut(cut));
        } else if let Some(caps) = inc_re.captures(line) {
            let inc = caps["inc"].parse::<i128>().expect("Malformed increment");
            shuffles.push(ShuffleType::Increment(inc));
        } else {
            panic!("Unexpected shuffle");
        }
    }

    shuffles
}

fn combine_input(num_cards: i128, input: &Vec<ShuffleType>) -> (i128, i128) {
    input.iter().fold((1, 0), |acc, shuffle| {
        let muladd = shuffle.to_multiply_add(num_cards);
        (
            (acc.0 * muladd.0) % num_cards,
            ((acc.1 * muladd.0) + muladd.1) % num_cards,
        )
    })
}

fn shuffle(num_cards: i128, input: &Vec<ShuffleType>, index: i128) -> i128 {
    let muladd = combine_input(num_cards, &input);
    r#mod(muladd.0 * index + muladd.1, num_cards)
}

fn reverse_shuffle(num_cards: i128, input: &Vec<ShuffleType>, index: i128) -> i128 {
    reverse_shuffle_repeat(num_cards, input, index, 1)
}

fn reverse_shuffle_repeat(
    num_cards: i128,
    input: &Vec<ShuffleType>,
    index: i128,
    repeat: i128,
) -> i128 {
    let mut input: Vec<ShuffleType> = input.iter().map(|s| s.inverse(num_cards)).collect();
    input.reverse();
    let muladd = combine_input(num_cards, &input);
    let muladd = repeat_shuffle(num_cards, muladd, repeat);
    r#mod(muladd.0 * index + muladd.1, num_cards)
}

fn repeat_shuffle(num_cards: i128, muladd: (i128, i128), repeat: i128) -> (i128, i128) {
    if repeat == 1 {
        muladd
    } else if repeat % 2 == 0 {
        repeat_shuffle(
            num_cards,
            (
                (muladd.0 * muladd.0) % num_cards,
                (muladd.0 * muladd.1 + muladd.1) % num_cards,
            ),
            repeat / 2,
        )
    } else {
        let (c, d) = repeat_shuffle(num_cards, muladd, repeat - 1);
        (
            (muladd.0 * c) % num_cards,
            (muladd.0 * d + muladd.1) % num_cards,
        )
    }
}

fn main() {
    let shuffles = parse_input("input");

    // Part 1
    const PT1_NUM_CARDS: i128 = 10007;
    const PT1_TGT_INDEX: i128 = 2019;

    let result = shuffle(PT1_NUM_CARDS, &shuffles, PT1_TGT_INDEX);
    println!("Card {} at index: {}", PT1_TGT_INDEX, result);

    // Part 2
    const PT2_SHUFFLE_COUNT: i128 = 101741582076661;
    const PT2_NUM_CARDS: i128 = 119315717514047;
    const PT2_TGT_INDEX: i128 = 2020;

    let result = reverse_shuffle_repeat(PT2_NUM_CARDS, &shuffles, PT2_TGT_INDEX, PT2_SHUFFLE_COUNT);
    println!("Card at index {}: {}", PT2_TGT_INDEX, result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_reverse() {
        let shuffles = vec![ShuffleType::Stack];
        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 8);

        let result = reverse_shuffle(10, &shuffles, 7);
        assert_eq!(result, 2);
    }

    #[test]
    fn cut_reverse() {
        let shuffles = vec![ShuffleType::Cut(3)];
        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 4);

        let result = reverse_shuffle(10, &shuffles, 9);
        assert_eq!(result, 2);
    }

    #[test]
    fn cut_negative_reverse() {
        let shuffles = vec![ShuffleType::Cut(-4)];

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 7);

        let result = reverse_shuffle(10, &shuffles, 9);
        assert_eq!(result, 5);
    }

    #[test]
    fn increment_reverse() {
        let shuffles = vec![ShuffleType::Increment(3)];
        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 7);

        let result = reverse_shuffle(10, &shuffles, 9);
        assert_eq!(result, 3);
    }

    #[test]
    fn ex1_reverse() {
        let shuffles = vec![
            ShuffleType::Increment(7),
            ShuffleType::Stack,
            ShuffleType::Stack,
        ];

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 3);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 8);
    }

    #[test]
    fn ex2_reverse() {
        let shuffles = vec![
            ShuffleType::Cut(6),
            ShuffleType::Increment(7),
            ShuffleType::Stack,
        ];

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 0);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 5);
    }

    #[test]
    fn ex3_reverse() {
        let shuffles = vec![
            ShuffleType::Increment(7),
            ShuffleType::Increment(9),
            ShuffleType::Cut(-2),
        ];

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 3);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 8);
    }

    #[test]
    fn ex4_reverse() {
        let shuffles = vec![
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
        ];

        let result = reverse_shuffle(10, &shuffles, 1);
        assert_eq!(result, 2);

        let result = reverse_shuffle(10, &shuffles, 6);
        assert_eq!(result, 7);
    }
}
