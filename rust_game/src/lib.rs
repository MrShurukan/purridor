#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

mod runtime;
mod raylib;
mod game;

use core::ffi::c_int;

use raylib::{
    App,
    Button,
    Color,
    Rect,
    Text,
    Vec2,
};
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::game::world::Game;

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

    while app.running() {
        let dt = app.delta_time();

        let mut frame =
            app.begin_frame(Color::rgb(10, 10, 10));

        game.process(dt);
        game.draw(&mut frame);

        // EndDrawing() happens automatically here.
    }

    // CloseWindow() automatically happens
    // when App is dropped.

    0
}