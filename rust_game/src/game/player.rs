use crate::game::util::CoordsConvertable;
use crate::game::world::{TILES_X, TILES_Y, TILE_SIZE};
use crate::raylib::{Color, Frame, Vec2};

pub struct Player {
    x: usize,
    y: usize,
}

impl Player {
    pub fn new() -> Self { Player { x: TILES_X / 2, y: TILES_Y } }

    pub fn draw(&self, frame: &mut Frame) {
        let x = self.x as f32 + 0.5;
        let y = self.y as f32 + 0.5;

        frame.circle((x, y).to_screen_coords(), (TILE_SIZE as f32) * 0.8, Color::YELLOW)
    }

    pub fn translate(&mut self, delta_x: i32, delta_y: i32) {
        self.x = ((self.x as i32) + delta_x).max(0) as usize;
        self.y = ((self.y as i32) + delta_y).max(0) as usize;
    }
}