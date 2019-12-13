extern crate intcode;

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::graphics::{Color, DrawMode, DrawParam, Mesh, Text};
use ggez::timer;
use ggez::{Context, GameResult};
use intcode::Program;
use std::cell::Cell;
use std::cmp::max;
use std::collections::HashMap;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

const TILE_WALL: i64 = 1;
const TILE_BLOCK: i64 = 2;
const TILE_PADDLE: i64 = 3;
const TILE_BALL: i64 = 4;

const INPUT_LEFT: i64 = -1;
const INPUT_NEUTRAL: i64 = 0;
const INPUT_RIGHT: i64 = 1;

const CLEAR_COLOR: Color = graphics::BLACK;
const WALL_COLOR: Color = graphics::WHITE;
const BLOCK_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const PADDLE_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
const BALL_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
const TEXT_COLOR: Color = graphics::BLACK;

enum OutputMode {
    SetX,
    SetY,
    Draw,
    Score,
}

struct Game {
    program: Program,
    screen: HashMap<(i64, i64), i64>,
    score: i64,
}

impl Game {
    fn new(filename: &str) -> Self {
        let mut program = Program::from_file(filename);

        // Set freeplay mode.
        program.poke(0, 2);

        Game {
            program: program,
            screen: HashMap::new(),
            score: 0,
        }
    }

    fn find_unique_tile(&self, find_type: i64) -> Option<(i64, i64)> {
        assert!(find_type == TILE_BALL || find_type == TILE_PADDLE);
        for (coords, tile_type) in self.screen.clone() {
            if tile_type == find_type {
                return Some(coords);
            }
        }

        None
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 200;

        if self.program.is_halted() {
            return Ok(());
        }

        if timer::check_update_time(ctx, DESIRED_FPS) {
            let mut x = 0;
            let mut y = 0;
            let mut output_mode = OutputMode::SetX;
            let mut screen = self.screen.clone();
            let mut score = self.score;

            let ball_loc_ref = Cell::new(self.find_unique_tile(TILE_BALL));
            let paddle_loc_ref = Cell::new(self.find_unique_tile(TILE_PADDLE));

            // Run the program until it asks for an input, give the input,
            // then take a break to do some drawing.
            let mut done = false;
            let mut result: Result<(), _> = Ok(());
            while !done && result.is_ok() {
                result = self.program.step(
                    &mut || {
                        let ball_coords = ball_loc_ref.get();
                        let paddle_coords = paddle_loc_ref.get();

                        let input = match (ball_coords, paddle_coords) {
                            (Some((ball_x, _)), Some((paddle_x, _))) => {
                                if ball_x > paddle_x {
                                    INPUT_RIGHT
                                } else if ball_x < paddle_x {
                                    INPUT_LEFT
                                } else {
                                    INPUT_NEUTRAL
                                }
                            }
                            _ => INPUT_NEUTRAL,
                        };

                        done = true;
                        input
                    },
                    &mut |val| {
                        match output_mode {
                            OutputMode::SetX => {
                                x = val;
                                output_mode = OutputMode::SetY;
                            }
                            OutputMode::SetY => {
                                y = val;

                                if x == -1 && y == 0 {
                                    output_mode = OutputMode::Score;
                                } else {
                                    output_mode = OutputMode::Draw;
                                }
                            }
                            OutputMode::Draw => {
                                screen.insert((x, y), val);

                                match val {
                                    TILE_BALL => ball_loc_ref.set(Some((x, y))),
                                    TILE_PADDLE => paddle_loc_ref.set(Some((x, y))),
                                    _ => (),
                                };

                                output_mode = OutputMode::SetX;
                            }
                            OutputMode::Score => {
                                score = val;
                                output_mode = OutputMode::SetX;
                            }
                        };
                    },
                );
            }

            self.screen = screen;

            if score != self.score {
                println!("Score: {}", score);
                self.score = score;
            }

            if result.is_err() {
                event::quit(ctx);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, CLEAR_COLOR);

        // Find the size of the screen that the program is drawing.
        let bounds = self
            .screen
            .iter()
            .fold((0, 0), |acc, ((x, y), _)| (max(acc.0, *x), max(acc.1, *y)));

        let block_width = SCREEN_WIDTH / ((1 + bounds.0) as f32);
        let block_height = SCREEN_HEIGHT / ((1 + bounds.1) as f32);

        for ((x, y), block) in self.screen.clone() {
            let left = x as f32 * block_width;
            let top = y as f32 * block_height;

            let color = match block {
                TILE_WALL => WALL_COLOR,
                TILE_BLOCK => BLOCK_COLOR,
                TILE_PADDLE => PADDLE_COLOR,
                TILE_BALL => BALL_COLOR,
                _ => CLEAR_COLOR,
            };

            let rect = graphics::Rect::new(left, top, block_width, block_height);
            let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }

        graphics::draw(
            ctx,
            &Text::new(format!("{}", self.score)),
            DrawParam::default().color(TEXT_COLOR),
        )?;

        graphics::present(ctx)
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("AOC19 - Day 13", "juzley")
        .window_setup(ggez::conf::WindowSetup::default().title("Breakout!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT));
    let (ctx, events_loop) = &mut cb.build().unwrap();
    let game = &mut Game::new("input");
    event::run(ctx, events_loop, game)
}
