use intcode::Program;

#[derive(Copy, Clone)]
enum Register {
    GroundOne,
    GroundTwo,
    GroundThree,
    GroundFour,
    GroundFive,
    GroundSix,
    GroundSeven,
    GroundEight,
    GroundNine,
    Temp,
    Jump,
}

impl Register {
    fn to_string(self) -> String {
        let s = match self {
            Register::GroundOne => "A",
            Register::GroundTwo => "B",
            Register::GroundThree => "C",
            Register::GroundFour => "D",
            Register::GroundFive => "E",
            Register::GroundSix => "F",
            Register::GroundSeven => "G",
            Register::GroundEight => "H",
            Register::GroundNine => "I",
            Register::Temp => "T",
            Register::Jump => "J",
        };

        String::from(s)
    }
}

#[derive(Copy, Clone)]
enum Command {
    Not(Register, Register),
    And(Register, Register),
    Or(Register, Register),
    Walk,
    Run,
}

impl Command {
    fn to_string(self) -> String {
        match self {
            Command::Not(o1, o2) => format!("{} {} {}", "NOT", o1.to_string(), o2.to_string()),
            Command::And(o1, o2) => format!("{} {} {}", "AND", o1.to_string(), o2.to_string()),
            Command::Or(o1, o2) => format!("{} {} {}", "OR", o1.to_string(), o2.to_string()),
            Command::Walk => String::from("WALK"),
            Command::Run => String::from("RUN"),
        }
    }
}

struct SpringScript(Vec<Command>);

impl SpringScript {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .cloned()
            .map(|c| format!("{}\n", c.to_string()))
            .collect::<Vec<String>>()
            .join("")
    }

    fn to_ascii(&self) -> Vec<u8> {
        self.to_string().chars().map(|c| c as u8).collect()
    }
}

fn execute_springscript(program: &Program, script: &SpringScript) -> Option<i64> {
    let buf = script.to_ascii();
    let mut input = buf.iter();
    let mut output = None;

    program.execute_ex(
        || {
            let inp = input.next().unwrap();
            print!("{}", *inp as char);
            *inp as i64
        },
        |v| {
            if v >= 128 {
                output = Some(v);
            } else {
                print!("{}", (v as u8) as char);
            }
        },
    );

    output
}

fn main() {
    let prg = Program::from_file("input");

    // Part 1
    let script = SpringScript(vec![
        // Jump = !(1 && 2 && 3) && 4
        Command::Or(Register::GroundOne, Register::Temp),
        Command::And(Register::GroundTwo, Register::Temp),
        Command::And(Register::GroundThree, Register::Temp),
        Command::Not(Register::Temp, Register::Jump),
        Command::And(Register::GroundFour, Register::Jump),
        // Reset the temp register
        Command::Not(Register::Jump, Register::Temp),
        Command::And(Register::Jump, Register::Temp),
        // Walk
        Command::Walk,
    ]);
    let damage = execute_springscript(&prg, &script);
    println!("Part 1 Damage: {}", damage.unwrap());

    // Part 2: Jump = !(1 && 2 && 3) && (5 || 8) && 4
    let script = SpringScript(vec![
        // A: !(1 && 2 && 3) -> Jump
        Command::Or(Register::GroundOne, Register::Temp),
        Command::And(Register::GroundTwo, Register::Temp),
        Command::And(Register::GroundThree, Register::Temp),
        Command::Not(Register::Temp, Register::Jump),
        // Reset the temp register
        Command::Not(Register::Jump, Register::Temp),
        Command::And(Register::Jump, Register::Temp),
        // B: (5 || 8) -> Temp
        Command::Or(Register::GroundFive, Register::Temp),
        Command::Or(Register::GroundEight, Register::Temp),
        // A && B && 4 -> Jump
        Command::And(Register::Temp, Register::Jump),
        Command::And(Register::GroundFour, Register::Jump),
        // Reset the temp register
        Command::Not(Register::Jump, Register::Temp),
        Command::And(Register::Jump, Register::Temp),
        // Run
        Command::Run,
    ]);
    let damage = execute_springscript(&prg, &script);
    println!("Part 2 Damage: {:?}", damage.unwrap());
}
