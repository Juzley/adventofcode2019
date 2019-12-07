use std::cmp;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

extern crate intcode;

fn make_permutations(input: Vec<u32>, permutation: Vec<u32>, permutations: &mut Vec<Vec<u32>>) {
    if input.is_empty() {
        permutations.push(permutation);
        return;
    }

    for i in 0..input.len() {
        let mut new_permutation = permutation.clone();
        new_permutation.push(input[i]);

        let mut new_input = input.clone();
        new_input.remove(i);

        make_permutations(new_input, new_permutation, permutations);
    }
}

fn part1() -> i32 {
    let program = intcode::Program::from_file("input");

    // Make all permutations of stage inputs.
    let mut permutations = Vec::new();
    make_permutations(vec![0, 1, 2, 3, 4], vec![], &mut permutations);

    let mut max_output = 0;

    let input_permutations = permutations;
    for input_perm in input_permutations {
        let mut stage_output = 0;

        // Execute each stage - the input is first taken from the
        // input permutation, and then the output from the previous stage.
        for input in input_perm {
            let input = [input as i32, stage_output];
            let mut input_iter = input.iter();
            program.execute_ex(
                || *input_iter.next().unwrap(),
                |output| stage_output = output,
            );
        }

        // Check if the output from the final stage was higher than
        // any previous permutation.
        max_output = cmp::max(stage_output, max_output);
    }

    return max_output;
}

fn spawn_amp(
    amp: intcode::Program,
    phase: u32,
    input: Option<i32>,
    tx: Sender<i32>,
    rx: Receiver<i32>,
) -> thread::JoinHandle<Option<i32>> {
    return thread::spawn(move || {
        let mut set_phase = false;
        let mut initial_input = input;
        let mut last_output = None;

        let input_fn = || {
            if !set_phase {
                set_phase = true;
                return phase as i32;
            } else if let Some(i) = initial_input {
                initial_input = None;
                return i;
            } else {
                return rx.recv().unwrap();
            }
        };

        let output_fn = |val| {
            last_output = Some(val);

            // The send may fail if the next amp has already
            // halted, just ignore.
            let _ = tx.send(val);
        };

        amp.execute_ex(input_fn, output_fn);
        return last_output;
    });
}

fn part2() -> i32 {
    // Make all permutations of stage inputs.
    let mut permutations = Vec::new();
    make_permutations(vec![5, 6, 7, 8, 9], vec![], &mut permutations);

    let amp_program = intcode::Program::from_file("input");

    let mut max_output = 0;
    for phases in permutations {
        // Need to connect the amplifiers together, such that output values from one
        // amplifier are used as the inputs for the next. Do this by running
        // each amplifier's program in a separate thread, and passing values
        // between the threads using channels.
        let (a_tx, b_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
        let (b_tx, c_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
        let (c_tx, d_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
        let (d_tx, e_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
        let (e_tx, a_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

        let mut prg_a = amp_program.clone();
        prg_a.set_name("Amplifier A");
        let mut prg_b = amp_program.clone();
        prg_b.set_name("Amplifier B");
        let mut prg_c = amp_program.clone();
        prg_c.set_name("Amplifier C");
        let mut prg_d = amp_program.clone();
        prg_d.set_name("Amplifier D");
        let mut prg_e = amp_program.clone();
        prg_e.set_name("Amplifier E");

        let amp_a = spawn_amp(prg_a, phases[0], Some(0), a_tx, a_rx);
        let amp_b = spawn_amp(prg_b, phases[1], None, b_tx, b_rx);
        let amp_c = spawn_amp(prg_c, phases[2], None, c_tx, c_rx);
        let amp_d = spawn_amp(prg_d, phases[3], None, d_tx, d_rx);
        let amp_e = spawn_amp(prg_e, phases[4], None, e_tx, e_rx);

        for amp in vec![amp_a, amp_b, amp_c, amp_d] {
            amp.join().expect("Amplifier failed");
        }

        let output = amp_e.join().expect("No output from final amplifier");
        max_output = cmp::max(max_output, output.unwrap());
    }

    return max_output;
}

fn main() {
    let result = part1();
    println!("Max linear output: {}", result);

    let result = part2();
    println!("Max feedback output: {}", result);
}
