use std::fs::File;
use std::io::{BufRead, BufReader};

fn get_pattern_elem(row: usize, column: usize) -> i8 {
    let column = column + 1; // We skip the first elem
    let row = row + 1; // Row is zero-indexed, but repeat count starts at 1.

    let key = [0, 1, 0, -1];

    // Index into the pattern (the extended key)
    let pattern_index = column % (row * key.len());

    // Index into the key
    let key_index = pattern_index / row;

    key[key_index]
}

fn get_input_elem(row: usize, input: &Vec<u8>) -> u8 {
    input[row % input.len()]
}

fn calc_digit(input: &Vec<u8>, input_len: usize, row: usize) -> u8 {
    let mut digit: i32 = 0;
    for i in row..input_len {
        let pattern_digit = get_pattern_elem(row, i);
        let input_digit = get_input_elem(i, input);

        // TODO: Maybe some mod stuff here to deal with overflow
        digit += pattern_digit as i32 * input_digit as i32;
    }

    (digit.abs() % 10) as u8
}

fn calc_phase(input: Vec<u8>) -> Vec<u8> {
    let mut output = Vec::new();

    for i in 0..input.len() {
        output.push(calc_digit(&input, input.len(), i));
    }

    output
}

fn calc_phases(input: Vec<u8>, phases: u32) -> Vec<u8> {
    let mut input = input;

    for _ in 0..phases {
        input = calc_phase(input);
    }

    input
}

fn split_input(line: &str) -> Vec<u8> {
    return line.trim().chars().map(|c| c.to_digit(10).unwrap() as u8).collect();
}

fn read_input(filename: &str) -> Vec<u8> {
    let file = File::open(filename).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read line");
    return split_input(line.as_ref());
}

fn main() {
    let input = read_input("input");
    let output = calc_phases(input, 100);
    let output_str = output.into_iter().take(8).map(|d| d.to_string()).collect::<Vec<_>>().join("");
    println!("Result: {}", output_str);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt1_ex1() {
        let result = calc_phases(vec![1, 2, 3, 4, 5, 6, 7, 8], 4);
        assert_eq!(result, vec![0, 1, 0, 2, 9, 4, 9, 8]);
    }

    #[test]
    fn pt1_ex2() {
        let input = split_input("80871224585914546619083218645595");
        let output = calc_phases(input, 100);
        let result = output.into_iter().take(8).collect::<Vec<u8>>();
        assert_eq!(result, vec![2, 4, 1, 7, 6, 1, 7, 6])

    }

    #[test]
    fn pt1_ex3() {
        let input = split_input("19617804207202209144916044189917");
        let output = calc_phases(input, 100);
        let result = output.into_iter().take(8).collect::<Vec<u8>>();
        assert_eq!(result, vec![7, 3, 7, 4, 5, 4, 1, 8]);
    }

    #[test]
    fn pt1_ex4() {
        let input = split_input("69317163492948606335995924319873");
        let output = calc_phases(input, 100);
        let result = output.into_iter().take(8).collect::<Vec<u8>>();
        assert_eq!(result, vec![5, 2, 4, 3, 2, 1, 3, 3]);
    }
}

