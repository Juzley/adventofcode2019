extern crate intcode;

fn main() {
    let program = intcode::Program::from_file("input");
    program.execute();
}
