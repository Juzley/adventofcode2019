use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

const OPCODE_ADD: i32 = 1;
const OPCODE_MUL: i32 = 2;
const OPCODE_IN: i32 = 3;
const OPCODE_OUT: i32 = 4;
const OPCODE_JIT: i32 = 5;
const OPCODE_JIF: i32 = 6;
const OPCODE_LT: i32 = 7;
const OPCODE_EQ: i32 = 8;
const OPCODE_HALT: i32 = 99;

#[derive(Copy, Clone, Debug)]
enum Operation {
    ADD,
    MUL,
    IN,
    OUT,
    JIT,
    JIF,
    LT,
    EQ,
    HALT,
}

#[derive(Copy, Clone, Debug)]
enum ParameterMode {
    POSITION,
    DIRECT,
}

#[derive(Debug)]
struct Instruction {
    op: Operation,
    param_modes: [ParameterMode; 3],
}

impl Instruction {
    fn from_int(raw: i32) -> Instruction {
        let get_param_mode = |slot: i32| {
            let base: i32 = 10;
            let exp: u32 = (slot + 2) as u32;
            return match (raw / base.pow(exp)) % 10 {
                1 => ParameterMode::DIRECT,
                _ => ParameterMode::POSITION,
            };
        };

        let raw_op = raw % 100;
        let op = match raw_op {
            OPCODE_ADD => Some(Operation::ADD),
            OPCODE_MUL => Some(Operation::MUL),
            OPCODE_IN => Some(Operation::IN),
            OPCODE_OUT => Some(Operation::OUT),
            OPCODE_JIT => Some(Operation::JIT),
            OPCODE_JIF => Some(Operation::JIF),
            OPCODE_LT => Some(Operation::LT),
            OPCODE_EQ => Some(Operation::EQ),
            OPCODE_HALT => Some(Operation::HALT),
            _ => None,
        };
        assert!(op.is_some(), "Unknown opcode {}", raw_op);
        return Instruction {
            op: op.unwrap(),
            param_modes: [get_param_mode(0), get_param_mode(1), get_param_mode(2)],
        };
    }
}

fn read(program: &Vec<i32>, param: i32, param_mode: ParameterMode) -> i32 {
    return match param_mode {
        ParameterMode::DIRECT => param,
        ParameterMode::POSITION => program[param as usize],
    };
}

fn write(program: &mut Vec<i32>, value: i32, position: i32) {
    program[position as usize] = value;
}

#[derive(Clone)]
pub struct Program {
    program: Vec<i32>,
    name: String,
}

impl Program {
    pub fn from_str(line: &str) -> Program {
        let strs: Vec<&str> = line.trim().split(",").collect();
        let instructions: Vec<i32> = strs
            .into_iter()
            .map(|s| s.parse::<i32>().expect("Failed to parse value"))
            .collect();

        return Program {
            program: instructions,
            name: String::new(),
        };
    }

    pub fn from_file(filename: &str) -> Program {
        let file = File::open(filename).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read line");
        return Program::from_str(line.as_ref());
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }

    pub fn execute(&self) {
        let input_fn = || {
            let mut val = None;
            while val.is_none() {
                println!("Provide Input:");
                let mut inp = String::new();
                io::stdin()
                    .read_line(&mut inp)
                    .expect("Failed to read line");
                val = inp.trim().parse::<i32>().ok();
            }

            return val.unwrap();
        };

        self.execute_ex(input_fn, |val| println!("Output: {}", val));
    }

    pub fn execute_ex<I, O>(&self, mut input_fn: I, mut output_fn: O)
    where
        I: FnMut() -> i32,
        O: FnMut(i32) -> (),
    {
        // Execution modifies the program buffer, so clone it first so we can execute
        // this program again if required.
        let mut program = self.program.clone();

        let mut i = 0;
        while i < program.len() {
            let instruction = Instruction::from_int(program[i]);
            i += 1;

            println!("{} {}, {:?}", self.name, i, instruction);

            let mut binary_op = |op_fn: &dyn Fn(i32, i32) -> i32| {
                let val1 = read(&program, program[i], instruction.param_modes[0]);
                let val2 = read(&program, program[i + 1], instruction.param_modes[1]);
                let dst = program[i + 2];
                write(&mut program, op_fn(val1, val2), dst);
                i += 3;
            };

            match instruction.op {
                Operation::ADD => binary_op(&|v1, v2| v1 + v2),
                Operation::MUL => binary_op(&|v1, v2| v1 * v2),
                Operation::LT => binary_op(&|v1, v2| if v1 < v2 { 1 } else { 0 }),
                Operation::EQ => binary_op(&|v1, v2| if v1 == v2 { 1 } else { 0 }),
                Operation::IN => {
                    let pos = program[i];
                    write(&mut program, input_fn(), pos);
                    i += 1;
                }
                Operation::OUT => {
                    let val = read(&program, program[i], instruction.param_modes[0]);
                    output_fn(val);
                    i += 1;
                }
                Operation::JIT => {
                    let val = read(&program, program[i], instruction.param_modes[0]);
                    let dst = read(&program, program[i + 1], instruction.param_modes[1]);
                    if val != 0 {
                        i = dst as usize;
                    } else {
                        i += 2;
                    }
                }
                Operation::JIF => {
                    let val = read(&program, program[i], instruction.param_modes[0]);
                    let dst = read(&program, program[i + 1], instruction.param_modes[1]);
                    if val == 0 {
                        i = dst as usize;
                    } else {
                        i += 2;
                    }
                }
                Operation::HALT => {
                    break;
                }
            }
        }
    }
}
