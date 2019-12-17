use std::fs::File;
use std::io::{BufRead, BufReader};

const OFFSET_LEN: usize = 7;
const INPUT_REPEAT: usize = 10000;

fn calc_phases(input: &Vec<u8>, phases: u32) -> Vec<u8> {
    let mut buf = input.clone();

    for _ in 0..phases {
        let mut sum = 0;
        for i in (0..input.len()).rev() {
            sum = (sum + buf[i]) % 10;
            buf[i] = sum;
        }
    }

    buf
}

fn extract_num(buf: &Vec<u8>, offset: usize, len: usize) -> u64 {
    let mut result = 0;
    for val in &buf[offset..(offset + len)] {
        result *= 10;
        result += *val as u64;
    }

    result
}

fn split_input(line: &str) -> Vec<u8> {
    return line
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();
}

fn read_input(filename: &str) -> Vec<u8> {
    let file = File::open(filename).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read line");
    return split_input(line.as_ref());
}

fn main() {
    // Part 1
    let input = read_input("input");
    let output = calc_phases(&input, 100);
    let result = extract_num(&output, 0, 8);
    println!("Part 1 Result: {}", result);

    // Part 2
    let offset = extract_num(&input, 0, OFFSET_LEN) as usize;

    let input_len = (INPUT_REPEAT * input.len()) - offset;
    let mut repeated_input = Vec::with_capacity(input_len);
    for i in 0..input_len {
        repeated_input.push(input[(i + offset) % input.len()]);
    }
    let output = calc_phases(&repeated_input, 100);
    let result = extract_num(&output, 0, 8);
    println!("Part 2 Result: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt1_ex1() {
        let output = calc_phases(&vec![1, 2, 3, 4, 5, 6, 7, 8], 4);
        let result = extract_num(&output, 0, 8);
        assert_eq!(result, 01029498);
    }

    #[test]
    fn pt1_ex2() {
        let input = split_input("80871224585914546619083218645595");
        let output = calc_phases(&input, 100);
        let result = extract_num(&output, 0, 8);
        assert_eq!(result, 24176176)
    }

    #[test]
    fn pt1_ex3() {
        let input = split_input("19617804207202209144916044189917");
        let output = calc_phases(&input, 100);
        let result = extract_num(&output, 0, 8);
        assert_eq!(result, 73745418);
    }

    #[test]
    fn pt1_ex4() {
        let input = split_input("69317163492948606335995924319873");
        let output = calc_phases(&input, 100);
        let result = extract_num(&output, 0, 8);
        assert_eq!(result, 52432133);
    }

    //#[test]
    fn pt2_e1() {
        /*let input = split_input("03036732577212944063491565474664");
        let offset = get_offset(&input);
        let result = calc_phases(&input, 100, input.len() * INPUT_REPEAT, offset, 8);
        assert_eq!(result, 84462026);*/
    }
}
