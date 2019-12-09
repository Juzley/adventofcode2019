use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

const OPCODE_ADD: i8 = 1;
const OPCODE_MUL: i8 = 2;
const OPCODE_IN: i8 = 3;
const OPCODE_OUT: i8 = 4;
const OPCODE_JIT: i8 = 5;
const OPCODE_JIF: i8 = 6;
const OPCODE_LT: i8 = 7;
const OPCODE_EQ: i8 = 8;
const OPCODE_BASE: i8 = 9;
const OPCODE_HALT: i8 = 99;

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
    BASE,
    HALT,
}

#[derive(Copy, Clone, Debug)]
enum ParameterMode {
    POSITION,
    DIRECT,
    RELATIVE,
}

#[derive(Debug)]
struct Instruction {
    op: Operation,
    params: Vec<i64>,
    param_modes: Vec<ParameterMode>,
}

impl Instruction {
    fn new(buf: &[i64], index: usize) -> Instruction {
        let get_param_mode = |slot: i32| {
            let base: i64 = 10;
            let exp: u32 = (slot + 2) as u32;
            return match (buf[index] / base.pow(exp)) % 10 {
                1 => ParameterMode::DIRECT,
                2 => ParameterMode::RELATIVE,
                _ => ParameterMode::POSITION,
            };
        };

        let raw_op = (buf[index] % 100) as i8;
        let (op, param_count) = match raw_op {
            OPCODE_ADD => (Operation::ADD, 3),
            OPCODE_MUL => (Operation::MUL, 3),
            OPCODE_IN => (Operation::IN, 1),
            OPCODE_OUT => (Operation::OUT, 1),
            OPCODE_JIT => (Operation::JIT, 2),
            OPCODE_JIF => (Operation::JIF, 2),
            OPCODE_LT => (Operation::LT, 3),
            OPCODE_EQ => (Operation::EQ, 3),
            OPCODE_BASE => (Operation::BASE, 1),
            OPCODE_HALT => (Operation::HALT, 0),
            _ => panic!("Unknown opcode: {}", raw_op),
        };

        let mut params = Vec::new();
        let mut modes = Vec::new();
        for i in 0..param_count {
            params.push(buf[index + i + 1]);
            modes.push(get_param_mode(i as i32));
        }

        return Instruction {
            op: op,
            params: params,
            param_modes: modes,
        };
    }
}

fn read(mem: &Vec<i64>, param: i64, param_mode: ParameterMode, base: i64) -> i64 {
    let addr;
    match param_mode {
        ParameterMode::DIRECT => return param,
        ParameterMode::POSITION => addr = param as usize,
        ParameterMode::RELATIVE => addr = (param + base) as usize,
    };

    // We're reading beyond the memory we've allocated - we don't need to allocate
    // until we try to write, as it would be initialized to 0; we can just return 0.
    if addr >= mem.len() {
        return 0;
    }
    return mem[addr];
}

fn write(mem: &mut Vec<i64>, value: i64, position: i64, param_mode: ParameterMode, base: i64) {
    let addr = match param_mode {
        ParameterMode::DIRECT => panic!("Attempt to write in direct mode"),
        ParameterMode::POSITION => position as usize,
        ParameterMode::RELATIVE => (position + base) as usize,
    };

    if addr >= mem.len() {
        mem.resize(addr + 1, 0);
    }
    mem[addr] = value;
}

#[derive(Clone)]
pub struct Program {
    program: Vec<i64>,
    name: String,
}

impl Program {
    pub fn from_str(line: &str) -> Program {
        let strs: Vec<&str> = line.trim().split(",").collect();
        let instructions: Vec<i64> = strs
            .into_iter()
            .map(|s| s.parse::<i64>().expect("Failed to parse value"))
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
                val = inp.trim().parse::<i64>().ok();
            }

            return val.unwrap();
        };

        self.execute_ex(input_fn, |val| println!("Output: {}", val));
    }

    pub fn execute_ex<I, O>(&self, mut input_fn: I, mut output_fn: O)
    where
        I: FnMut() -> i64,
        O: FnMut(i64) -> (),
    {
        // Execution modifies the program buffer, so clone it first so we can execute
        // this program again if required.
        let mut mem = self.program.clone();
        let mut base = 0;

        let mut i = 0;
        while i < self.program.len() {
            let instruction = Instruction::new(&mem, i);
            println!("{} {}, {:?}", self.name, i, instruction);
            i += 1;

            let mut binary_op = |op_fn: &dyn Fn(i64, i64) -> i64, base: i64| {
                let val1 = read(
                    &mem,
                    instruction.params[0],
                    instruction.param_modes[0],
                    base,
                );
                let val2 = read(
                    &mem,
                    instruction.params[1],
                    instruction.param_modes[1],
                    base,
                );
                write(
                    &mut mem,
                    op_fn(val1, val2),
                    instruction.params[2],
                    instruction.param_modes[2],
                    base,
                );
                i += 3;
            };

            match instruction.op {
                Operation::ADD => binary_op(&|v1, v2| v1 + v2, base),
                Operation::MUL => binary_op(&|v1, v2| v1 * v2, base),
                Operation::LT => binary_op(&|v1, v2| if v1 < v2 { 1 } else { 0 }, base),
                Operation::EQ => binary_op(&|v1, v2| if v1 == v2 { 1 } else { 0 }, base),
                Operation::IN => {
                    write(
                        &mut mem,
                        input_fn(),
                        instruction.params[0],
                        instruction.param_modes[0],
                        base,
                    );
                    i += 1;
                }
                Operation::OUT => {
                    let val = read(
                        &mem,
                        instruction.params[0],
                        instruction.param_modes[0],
                        base,
                    );
                    output_fn(val);
                    i += 1;
                }
                Operation::JIT => {
                    let val = read(
                        &mem,
                        instruction.params[0],
                        instruction.param_modes[0],
                        base,
                    );
                    let dst = read(
                        &mem,
                        instruction.params[1],
                        instruction.param_modes[1],
                        base,
                    );
                    if val != 0 {
                        i = dst as usize;
                    } else {
                        i += 2;
                    }
                }
                Operation::JIF => {
                    let val = read(
                        &mem,
                        instruction.params[0],
                        instruction.param_modes[0],
                        base,
                    );
                    let dst = read(
                        &mem,
                        instruction.params[1],
                        instruction.param_modes[1],
                        base,
                    );
                    if val == 0 {
                        i = dst as usize;
                    } else {
                        i += 2;
                    }
                }
                Operation::BASE => {
                    let val = read(
                        &mem,
                        instruction.params[0],
                        instruction.param_modes[0],
                        base,
                    );
                    base += val;
                    i += 1;
                }
                Operation::HALT => {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_test() {
        // IO test from day 5 pt 1
        let prg = Program::from_str("3,0,4,0,99");

        let mut output = None;
        prg.execute_ex(|| 1, |val| output = Some(val));
        assert_eq!(output, Some(1));
    }

    #[test]
    fn test_eq_position() {
        // Eq with positional addressing from day 5 pt 2
        let prg = Program::from_str("3,9,8,9,10,9,4,9,99,-1,8");

        let mut output = None;
        prg.execute_ex(|| 8, |val| output = Some(val));
        assert_eq!(output, Some(1));

        let mut output = None;
        prg.execute_ex(|| 7, |val| output = Some(val));
        assert_eq!(output, Some(0));
    }

    #[test]
    fn test_lt_position() {
        // Less-than with positional addressing test from day 5 pt 2
        let prg = Program::from_str("3,9,7,9,10,9,4,9,99,-1,8");

        let mut output = None;
        prg.execute_ex(|| 8, |val| output = Some(val));
        assert_eq!(output, Some(0));

        let mut output = None;
        prg.execute_ex(|| 7, |val| output = Some(val));
        assert_eq!(output, Some(1));
    }

    #[test]
    fn test_eq_direct() {
        // Eq with direct addressing from day 5 pt 2
        let prg = Program::from_str("3,3,1108,-1,8,3,4,3,99");

        let mut output = None;
        prg.execute_ex(|| 8, |val| output = Some(val));
        assert_eq!(output, Some(1));

        let mut output = None;
        prg.execute_ex(|| 7, |val| output = Some(val));
        assert_eq!(output, Some(0));
    }

    #[test]
    fn test_lt_direct() {
        // Less-than with direct addressing test from day 5 pt 2
        let prg = Program::from_str("3,3,1107,-1,8,3,4,3,99");

        let mut output = None;
        prg.execute_ex(|| 8, |val| output = Some(val));
        assert_eq!(output, Some(0));

        let mut output = None;
        prg.execute_ex(|| 7, |val| output = Some(val));
        assert_eq!(output, Some(1));
    }

    #[test]
    fn jump_position() {
        // Jump with positional addressing test from day 5 pt 2
        let prg = Program::from_str("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9");

        let mut output = None;
        prg.execute_ex(|| 0, |val| output = Some(val));
        assert_eq!(output, Some(0));

        prg.execute_ex(|| 1, |val| output = Some(val));
        assert_eq!(output, Some(1));
    }

    #[test]
    fn jump_direct() {
        // Jump with direct addressing test from day 5 pt 2
        let prg = Program::from_str("3,3,1105,-1,9,1101,0,0,12,4,12,99,1");

        let mut output = None;
        prg.execute_ex(|| 0, |val| output = Some(val));
        assert_eq!(output, Some(0));

        prg.execute_ex(|| 1, |val| output = Some(val));
        assert_eq!(output, Some(1));
    }

    #[test]
    fn quine() {
        // Quine test from day 9 pt 1
        let prg_str = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        let prg = Program::from_str(prg_str);

        let mut output = Vec::new();
        prg.execute_ex(|| 0, |val| output.push(val));

        let output_strs: Vec<String> = output.iter().map(|v| v.to_string()).collect();
        let output_str = output_strs.join(",");
        assert_eq!(prg_str, output_str);
    }

    #[test]
    fn large_mul() {
        // Large number multiplication test from day 9 pt 1
        let prg = Program::from_str("1102,34915192,34915192,7,4,7,99,0");

        let mut output = None;
        prg.execute_ex(|| 0, |val| output = Some(val));

        assert_eq!(output, Some(34915192 * 34915192));
    }

    #[test]
    fn large_num() {
        // Large number test from day 9 pt 1
        let prg = Program::from_str("104,1125899906842624,99");

        let mut output = None;
        prg.execute_ex(|| 0, |val| output = Some(val));

        assert_eq!(output, Some(1125899906842624));
    }
}
