use crate::game::util::CoordsConvertable;
use crate::game::world::{TILES_DIM, TILE_SIZE};
use crate::raylib::{Color, Frame, Vec2};

pub enum PlayerSide {
    White,
    Black,
}

impl PlayerSide {
    pub fn index(&self) -> usize {
        match self {
            PlayerSide::White => 0,
            PlayerSide::Black => 1,
        }
    }

    pub fn other(&self) -> PlayerSide {
        match self {
            PlayerSide::White => PlayerSide::Black,
            PlayerSide::Black => PlayerSide::White,
        }
    }
}

pub struct Player {
    x: usize,
    y: usize,
    side: PlayerSide,

    available_walls: usize,
}

const PAWN_RADIUS: f32 = 20.0;
const PAWN_SHADOW_RADIUS: f32 = PAWN_RADIUS * 1.2;

impl Player {
    pub fn new(side: PlayerSide) -> Self {
        Player {
            x: 4,
            // White starts on the bottom, black on top
            y: if let PlayerSide::White = side { TILES_DIM - 1 } else { 0 },
            side,

            available_walls: 20,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let color = match self.side {
            PlayerSide::White => Color::rgb(220, 220, 220),
            PlayerSide::Black => Color::rgb(30, 30, 30),
        };

        let x = self.x as f32 + 0.5;
        let y = self.y as f32 + 0.5;

        // "Shadow"
        frame.circle((x, y).to_screen_coords(), PAWN_SHADOW_RADIUS, Color::rgba(30, 30, 30, 100));
        // Pawn
        frame.circle((x, y).to_screen_coords(), PAWN_RADIUS, color);
    }

    pub fn translate(&mut self, delta_x: i32, delta_y: i32) {
        self.x = ((self.x as i32) + delta_x).max(0) as usize;
        self.y = ((self.y as i32) + delta_y).max(0) as usize;
    }
}