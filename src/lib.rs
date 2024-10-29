extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{Key, RenderArgs,  UpdateArgs};

use rand::Rng;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Food {
    body_size: f64,
    position: Position,
}
impl Food {
    fn new() -> Food {
        let mut rng = rand::thread_rng();
        let body_size = 0.1;
        Food {
            body_size,
            position: Position {
                x: rng.gen_range(-10..10) as f64 * body_size,
                y: rng.gen_range(-10..10) as f64 * body_size,
        }
    }
    }
}

pub struct Snake {
    length: u32,
    direction: Direction,
    requested_direction: Direction,
    body_size: f64,
    body_segments: VecDeque<Position>,
}

impl Snake {
    pub fn new() -> Snake {
        Snake {
            length: 4,
            direction: Direction::Up,
            requested_direction: Direction::Up,
            body_size: 0.1,
            body_segments: VecDeque::from(vec![
                Position { x: 0.0, y: 0.0 },
                Position { x: 0.0, y: -0.1 },
                Position { x: 0.0, y: -0.2 },
                Position { x: 0.0, y: -0.3 },
            ]),
        }
    }

    fn update_position(&mut self) {
        match self.requested_direction {
            Direction::Up => {
                if self.direction != Direction::Down  { self.direction = self.requested_direction.clone()}
            }
            Direction::Down => {
                if self.direction != Direction::Up { self.direction = self.requested_direction.clone()}
            }
            Direction::Left => {
                if self.direction != Direction::Right { self.direction = self.requested_direction.clone()}
            }
            Direction::Right => {
                if self.direction != Direction::Left { self.direction = self.requested_direction.clone()}
            }
        }
        
        let mut y = self.body_segments[0].y;
        let mut x = self.body_segments[0].x;
        match self.direction {
            Direction::Up => {

                y = self.body_size + self.body_segments[0].y;
                if y > 0.95 {
                    y = 1.0
                }
            }
            Direction::Down => {
                y = self.body_segments[0].y - self.body_size;
                if y < -0.95 + self.body_size {
                    y = -1.0 + self.body_size
                } //taking account of the snake going past the window
            }
            Direction::Left => {
                x = self.body_segments[0].x - self.body_size;
                if x < -0.95 {
                    x = -1.0
                }
            }
            Direction::Right => {
                x = self.body_size + self.body_segments[0].x;
                if x > 0.95 - self.body_size {
                    x = 1.0 - self.body_size
                }
            }
        }

        dbg!(x,y);
        self.body_segments.push_front(Position {
            x,
            y,
        });
        self.body_segments.pop_back();
    }
}

pub struct SnakeGame {
    gl: GlGraphics, // OpenGL drawing backend.
    snake: Arc<Mutex<Snake>>,
}
impl SnakeGame {
    pub fn new(opengl: OpenGL) -> SnakeGame {
        SnakeGame {
            gl: GlGraphics::new(opengl),
            snake: Arc::new(Mutex::new(Snake::new())),
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREY: [f32; 4] = [0.502, 0.502, 0.502, 1.0];
        const FOREST_GREEN: [f32; 4] = [0.133, 0.545, 0.133, 1.0];

        let win_height = args.window_size[0];
        let win_width = args.window_size[1];

        let snake = self.snake.lock().unwrap();
        let square = rectangle::square(0.0, 0.0, ndc_to_pixel_length(snake.body_size, win_height));


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREY, gl);
            
            for element in snake.body_segments.iter() {
                let (x,y) = element.to_pixel_pos(win_height, win_width);
            
                let transform = c
                    .transform
                    .trans(x, y)
                    .trans(snake.body_size / 2.0, snake.body_size / 2.0);

                rectangle(FOREST_GREEN, square, transform, gl);
            }
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {}

    pub fn handle_key_press(&mut self, key: Key) {
        let mut snake = self.snake.lock().unwrap();
        match key {
            Key::Up => {
                snake.requested_direction = Direction::Up;
            }
            Key::Down => {
                snake.requested_direction = Direction::Down;
            }
            Key::Left => {
                snake.requested_direction = Direction::Left;
            }
            Key::Right => {
                snake.requested_direction = Direction::Right;
            }
            _ => {}
        }
    }

    pub fn start(&mut self) {
        let snake_clone = Arc::clone(&self.snake);
        thread::spawn(move || loop {
            {
                let mut snake = snake_clone.lock().unwrap();
                snake.update_position();
            }
            thread::sleep(Duration::from_millis(200));
        });
    }
}

struct Position {
    x: f64,
    y: f64,
}
impl Position {
    fn to_pixel_pos(&self, win_height: f64, win_width: f64) -> (f64, f64) {
        (
            (self.x * win_width / 2.0) + (win_width / 2.0),
            win_height - ((self.y * win_height / 2.0) + (win_height / 2.0))
        )
    }
}

fn ndc_to_pixel_length(ndc_length: f64, win_length: f64) -> f64 {
    ndc_length / 2.0 * win_length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_pixel_pos() {
        let pos = Position { x: 0.4, y:-0.8 };
        let (pixel_x, pixel_y) = pos.to_pixel_pos(1000.0,1000.0);
        assert_eq!(pixel_x, 700.0);
        assert_eq!(pixel_y, 900.0);
    }
}