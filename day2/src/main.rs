use std::fs::File;
use std::io::{BufRead, BufReader};

const OPCODE_ADD: usize = 1;
const OPCODE_MUL: usize = 2;

const MIN_INPUT: usize = 0;
const MAX_INPUT: usize = 99;

const TARGET_OUTPUT: usize = 19690720;

fn get_program(filename: &str) -> Vec<usize> {
    let file = File::open(filename).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read line");
    let strs: Vec<&str> = line.trim().split(",").collect();
    let prg: Vec<usize> = strs
        .into_iter()
        .map(|s| { s.parse::<usize>().expect("Failed to parse value") })
        .collect();

    return prg;
}

fn execute_program(program: &mut Vec<usize>) {
    let mut i = 0;
    while i + 3 < program.len() {
        let op = program[i];
        i += 1;
        let src1 = program[i];
        i += 1;
        let src2 = program[i];
        i += 1;
        let dst = program [i];
        i += 1;

        if op == OPCODE_ADD {
            program[dst] = program[src1] + program[src2];
        } else if op == OPCODE_MUL {
            program[dst] = program[src1] * program[src2];
        }
    }
}

fn set_input(program: &mut Vec<usize>, noun: usize, verb: usize) {
    program[1] = noun;
    program[2] = verb;
}

fn main() {
    let orig_prg = get_program("input");

    for n in MIN_INPUT..=MAX_INPUT {
        for v in MIN_INPUT..=MAX_INPUT {
            let mut prg = orig_prg.clone();
            set_input(&mut prg, n, v);
            execute_program(&mut prg);

            let output = prg[0];

            if output == TARGET_OUTPUT {
                println!("Found inputs! Noun: {noun}, Verb: {verb}", noun=n, verb=v);
                return;
            }
        }
    }

    println!("Didn't find inputs!");
    return;
}
