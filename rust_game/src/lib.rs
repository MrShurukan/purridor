#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod runtime;
mod raylib;
mod game;
mod panic_buffer;

use core::ffi::c_int;

use crate::game::input::GameInput;
use crate::game::world::Game;
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::Gamepad;
use raylib::{
    App,
    Color,
};

#[no_mangle]
pub extern "C" fn rust_main() -> c_int {
    let Ok(mut app) =
        App::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Rust Switch",
        )
    else {
        return 1;
    };

    app.set_target_fps(60);

    let mut game = Game::new();
    let gamepad = Gamepad::new();

    while app.running() {
        let dt = app.delta_time();

        let mut frame =
            app.begin_frame(Color::rgb(10, 10, 10));

        let input = GameInput::read(&gamepad);

        game = game.update(&input, dt);
        game.draw(&input, &mut frame);

        // EndDrawing() happens automatically here.
    }

    // CloseWindow() automatically happens
    // when App is dropped.

    0
}