extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::OpenGL;
use piston::event_loop::{EventSettings, Events};
use piston::input::{Button, PressEvent, RenderEvent};
use piston::window::WindowSettings;

use snake::SnakeGame;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("snake", [1000, 1000])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut snake_game = SnakeGame::new(opengl);

    snake_game.start();
    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            snake_game.render(&args);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            snake_game.handle_key_press(key);
        }
    }
}
