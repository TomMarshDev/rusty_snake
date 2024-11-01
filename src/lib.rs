extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;

use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::input::{Key, RenderArgs};

use rand::Rng;

use image::GenericImageView;

use std::collections::VecDeque;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Position {
    x: f64,
    y: f64,
}
impl Position {
    fn to_pixel_pos(&self, win_height: f64, win_width: f64) -> (f64, f64) {
        (
            (self.x * win_width / 2.0) + (win_width / 2.0),
            win_height - ((self.y * win_height / 2.0) + (win_height / 2.0)),
        )
    }

    fn to_pixel_length(&self, win_height: f64, win_width: f64) -> (f64, f64) {
        ((self.x / win_width * 2.0), (self.y / win_height * 2.0))
    }

    fn approx_eq(&self, other: &Position, tolerance: f64) -> bool {
        (self.x - other.x).abs() < tolerance && (self.y - other.y).abs() < tolerance
    }
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
            },
        }
    }

    fn update_position(&mut self) {
        let mut rng = rand::thread_rng();
        self.position.x = rng.gen_range(-10..10) as f64 * self.body_size;
        self.position.y = rng.gen_range(-10..10) as f64 * self.body_size;
    }
}

struct BodySegment {
    position: Position,
    direction: Direction
}
pub struct Snake {
    requested_direction: Direction,
    body_size: f64,
    body_segments: VecDeque<BodySegment>,
    pop_next_update: bool,
    head_texture_path: &'static str,
    tail_texture_path: &'static str,
    body_texture_path: &'static str,
}

impl Snake {
    pub fn new() -> Snake {
        Snake {
            requested_direction: Direction::Up,
            body_size: 0.1,
            body_segments: VecDeque::from(vec![
                BodySegment { position: Position {x: 0.0, y: 0.0}, direction: Direction::Up },
                BodySegment { position: Position {x: 0.0, y: -0.1}, direction: Direction::Up },
                BodySegment { position: Position {x: 0.0, y: -0.2}, direction: Direction::Up },
                // Position { x: 0.0, y: -0.3 },
            ]),
            pop_next_update: true,
            head_texture_path: "../assets/head.png",
            tail_texture_path: "../assets/tail.png",
            body_texture_path: "../assets/body.png",
        }
    }

    fn update_position(&mut self) {
        let mut pos = &self.body_segments[0].direction;
        match self.requested_direction {
            Direction::Up => {
                if self.body_segments[0].direction != Direction::Down {
                    pos = &Direction::Up;
                }
            }
            Direction::Down => {
                if self.body_segments[0].direction != Direction::Up {
                    pos = &Direction::Down;
                }
            }
            Direction::Left => {
                if self.body_segments[0].direction != Direction::Right {
                    pos = &Direction::Left;
                }
            }
            Direction::Right => {
                if self.body_segments[0].direction != Direction::Left {
                    pos = &Direction::Right;
                }
            }
        }

        let mut y = self.body_segments[0].position.y;
        let mut x = self.body_segments[0].position.x;
        match pos {
            Direction::Up => {
                y = self.body_size + self.body_segments[0].position.y;
                if y > 1.0 - 0.01 {
                    y = 1.0
                }
            }
            Direction::Down => {
                y = self.body_segments[0].position.y - self.body_size;
                if y < -1.0 + 0.01{
                    y = -1.0
                }
            }
            Direction::Left => {
                x = self.body_segments[0].position.x - self.body_size;
                if x < -1.0 + 0.01{
                    x = -1.0
                }
            }
            Direction::Right => {
                x = self.body_size + self.body_segments[0].position.x;
                if x > 1.0 - 0.01 {
                    x = 1.0
                }
            }
        }

        self.body_segments.push_front(BodySegment {position: Position { x, y }, direction: pos.clone()});

        if self.pop_next_update {
            self.body_segments.pop_back();
        }
        self.pop_next_update = true;
    }
}

pub struct SnakeGame {
    gl: GlGraphics, // OpenGL drawing backend.
    snake: Arc<Mutex<Snake>>,
    food: Arc<Mutex<Food>>,
    textures: Vec<Texture>,
}
impl SnakeGame {
    pub fn new(opengl: OpenGL) -> SnakeGame {
        SnakeGame {
            gl: GlGraphics::new(opengl),
            snake: Arc::new(Mutex::new(Snake::new())),
            food: Arc::new(Mutex::new(Food::new())),
            textures: Vec::new(),
        }
    }

    pub fn load_assets(&mut self) {
        let out_dir = env::var("OUT_DIR").unwrap();
        let assets_dir = PathBuf::from(out_dir).join("../../../assets");

        let snake = self.snake.lock().unwrap();

        let texture_paths = vec![
            assets_dir.join(snake.head_texture_path),
            assets_dir.join(snake.tail_texture_path),
            assets_dir.join(snake.body_texture_path),
        ];

        for path in texture_paths {
            let texture =
                Texture::from_path(&path, &TextureSettings::new()).expect("Failed to load texture");
            self.textures.push(texture);
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREY: [f32; 4] = [0.502, 0.502, 0.502, 1.0];
        const FOREST_GREEN: [f32; 4] = [0.133, 0.545, 0.133, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let win_height = args.window_size[0];
        let win_width = args.window_size[1];

        let snake = self.snake.lock().unwrap();
        // let square = rectangle::square(0.0, 0.0, ndc_to_pixel_length(&snake.body_size, &win_height));

        let food = self.food.lock().unwrap();
        let circle_radius = ndc_to_pixel_length(&food.body_size, &win_height) / 2.0;
        let circle = ellipse::circle(0.0, 0.0, circle_radius);
        
        let snake_length = snake.body_segments.len();

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREY, gl);
            
            //render snake
            for (i, element) in snake.body_segments.iter().enumerate().rev() {
                let (x, y) = element.position.to_pixel_pos(win_height, win_width);

                // Apply rotation
                let rotation_angle = match element.direction {
                    Direction::Up => 0.0,
                    Direction::Right => std::f64::consts::FRAC_PI_2,
                    Direction::Down => std::f64::consts::PI,
                    Direction::Left => -std::f64::consts::FRAC_PI_2,
                };

                // rotation multiplication
                let (x_rotation_factor, y_rotation_factor) = match element.direction {
                    Direction::Up => (0.0, 0.0),
                    Direction::Right => (0.0, -1.0),
                    Direction::Down => (-1.0, -1.0),
                    Direction::Left => (-1.0, 0.0),
                };

                // Calculate the center of the rectangle
                let center_x = snake.body_size / 2.0;
                let center_y = snake.body_size / 2.0;

                let pos_transform = c.transform.trans(x,y).trans(ndc_to_pixel_length(&center_x, &win_width) * -1.0, ndc_to_pixel_length(&center_y, &win_height) * -1.0);
                
                let rot_transform = pos_transform.rot_rad(rotation_angle).trans(ndc_to_pixel_length(&snake.body_size, &win_width) * x_rotation_factor, ndc_to_pixel_length(&snake.body_size, &win_height) * y_rotation_factor);
                
                // Apply scaling
                let scale_transform = rot_transform.scale(calculate_texture_scale(&snake.body_size, &win_width, &225.0), calculate_texture_scale(&snake.body_size, &win_height, &225.0));

                if i == 0 {
                    image(&self.textures[0], scale_transform, gl);
                } else if i == (snake_length -1){
                    image(&self.textures[1], scale_transform, gl);
                } else {
                    image(&self.textures[2], scale_transform, gl);
                }
            }

            // Render food as a circle
            let (food_x, food_y) = food.position.to_pixel_pos(win_height, win_width);
            let food_transform = c
                .transform
                .trans(food_x, food_y).trans(ndc_to_pixel_length(&(food.body_size / 2.0), &win_width) * -1.0, ndc_to_pixel_length(&(&food.body_size / 2.0), &win_height) * -1.0);


            ellipse(RED, circle, food_transform, gl);
        });
    }

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
        self.load_assets();

        let snake_clone = Arc::clone(&self.snake);
        let food_clone = Arc::clone(&self.food);
        thread::spawn(move || loop {
            {
                let mut snake = snake_clone.lock().unwrap();
                snake.update_position();

                let mut food = food_clone.lock().unwrap();
                if snake.body_segments[0].position.approx_eq(&food.position, 0.001) {
                    food.update_position();
                    snake.pop_next_update = false;
                }
            }
            thread::sleep(Duration::from_millis(300));
        });
    }
}

fn ndc_to_pixel_length(ndc_length: &f64, win_length: &f64) -> f64 {
    ndc_length / 2.0 * win_length
}

fn calculate_texture_scale(ndc_desired_length: &f64, win_pixel_length: &f64, texture_pixel_length: &f64) -> f64 {
    ((win_pixel_length / 2.0) * ndc_desired_length) / texture_pixel_length

}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_pixel_pos() {
        let pos = Position { x: 0.4, y: -0.8 };
        let (pixel_x, pixel_y) = pos.to_pixel_pos(1000.0, 1000.0);
        assert_eq!(pixel_x, 700.0);
        assert_eq!(pixel_y, 900.0);
    }

    #[test]
    fn test_calculate_texture_scale() {
        let ndc_desired_length = 0.1;
        let win_pixel_length = 1000.0;
        let texture_pixel_length = 250.0;
        assert_eq!(calculate_texture_scale(&ndc_desired_length, &win_pixel_length, &texture_pixel_length), 0.2)
    }
}
