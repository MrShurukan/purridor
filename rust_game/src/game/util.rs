use crate::game::player::Player;
use crate::game::world::{OFFSET_LEFT, OFFSET_TOP, TILE_SIZE};
use crate::raylib::Vec2;

pub trait CoordsConvertable {
    fn to_screen_coords(&self) -> Vec2;
    fn to_tile_coords(&self) -> Vec2;
}

impl CoordsConvertable for (f32, f32) {
    fn to_screen_coords(&self) -> Vec2 {
        Vec2::new(
            self.0 * TILE_SIZE as f32 + OFFSET_LEFT as f32,
            self.1 * TILE_SIZE as f32 + OFFSET_TOP as f32
        )
    }

    fn to_tile_coords(&self) -> Vec2 {
        Vec2::new(
            (self.0 - OFFSET_LEFT as f32) / TILE_SIZE as f32,
            (self.1 - OFFSET_TOP as f32) / TILE_SIZE as f32
        )
    }
}