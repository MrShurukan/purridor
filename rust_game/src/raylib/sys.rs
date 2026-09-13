#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int};

use super::{Color, Rect, Vec2};

unsafe extern "C" {
    // Window
    pub fn InitWindow(
        width: c_int,
        height: c_int,
        title: *const c_char,
    );

    pub fn CloseWindow();

    pub fn WindowShouldClose() -> bool;
    pub fn IsWindowReady() -> bool;

    // Random
    pub fn GetRandomValue(min: c_int, max: c_int) -> c_int;

    // Timing
    pub fn SetTargetFPS(fps: c_int);
    pub fn GetFrameTime() -> f32;

    // Drawing lifecycle
    pub fn BeginDrawing();
    pub fn EndDrawing();

    pub fn ClearBackground(color: Color);

    // Shapes
    pub fn DrawLineEx(
        start: Vec2,
        end: Vec2,
        thickness: f32,
        color: Color,
    );

    pub fn DrawRectangleRec(
        rect: Rect,
        color: Color,
    );

    pub fn DrawRectanglePro(
        rect: Rect,
        origin: Vec2,
        rotation: c_float,
        color: Color
    );

    pub fn DrawCircleV(
        center: Vec2,
        radius: f32,
        color: Color,
    );

    // Text
    pub fn DrawText(
        text: *const c_char,
        x: c_int,
        y: c_int,
        font_size: c_int,
        color: Color,
    );

    pub fn MeasureText(
        text: *const c_char,
        font_size: c_int,
    ) -> c_int;

    // Gamepad
    pub fn IsGamepadAvailable(gamepad: c_int) -> bool;

    pub fn IsGamepadButtonPressed(
        gamepad: c_int,
        button: c_int,
    ) -> bool;

    pub fn IsGamepadButtonDown(
        gamepad: c_int,
        button: c_int,
    ) -> bool;

    pub fn IsGamepadButtonReleased(
        gamepad: c_int,
        button: c_int,
    ) -> bool;

    pub fn GetGamepadAxisMovement(
        gamepad: c_int,
        axis: c_int,
    ) -> f32;
}