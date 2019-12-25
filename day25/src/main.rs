use intcode::Program;
use std::io::Read;

fn main() {
    let mut prg = Program::from_file("input");

    loop {
        let _ = prg.step(
            &mut || {
                let input: i64 = std::io::stdin()
                    .lock()
                    .bytes()
                    .next()
                    .and_then(|result| result.ok())
                    .map(|byte| byte as i64)
                    .unwrap();
                input
            },
            &mut |val| print!("{}", (val as u8) as char),
        );
    }
}
