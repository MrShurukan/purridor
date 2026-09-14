use crate::game::tile::TilePos;
use crate::game::world::TILES_DIM;
use crate::raylib::{Color, Frame};
use libm::sinf;
use crate::game::util::SinePulser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSide {
    White,
    Black,
}

impl PlayerSide {
    pub const fn index(self) -> usize {
        match self {
            PlayerSide::White => 0,
            PlayerSide::Black => 1,
        }
    }

    pub const fn other(self) -> PlayerSide {
        match self {
            PlayerSide::White => PlayerSide::Black,
            PlayerSide::Black => PlayerSide::White,
        }
    }

    pub const fn color(self) -> Color {
        match self {
            PlayerSide::White => Color::rgb(220, 220, 220),
            PlayerSide::Black => Color::rgb(30, 30, 30),
        }
    }
}

pub struct Player {
    pub pos: TilePos,
    side: PlayerSide,

    pub available_walls: usize,
}

const PAWN_RADIUS: f32 = 20.0;
const PAWN_SHADOW_RADIUS: f32 = PAWN_RADIUS * 1.2;

impl Player {
    pub fn new(side: PlayerSide) -> Self {
        Player {
            pos: TilePos {
                x: 4,
                y: if let PlayerSide::White = side { TILES_DIM - 1 } else { 0 }
            },
            side,

            available_walls: 20,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let color = self.side.color();

        // "Shadow"
        frame.circle(self.pos.screen_center(), PAWN_SHADOW_RADIUS, Color::rgba(30, 30, 30, 100));
        // Pawn
        frame.circle(self.pos.screen_center(), PAWN_RADIUS, color);
    }

    const GHOST_ALPHA_PULSER: SinePulser = SinePulser::new(20.0, 100.0, 2.0);

    pub fn draw_ghost(&self, location: &TilePos, frame: &mut Frame, time: f32) {
        let color = self.side.color();

        let alpha_pulse = Self::GHOST_ALPHA_PULSER.pulse(time);

        // "Shadow"
        frame.circle(location.screen_center(), PAWN_SHADOW_RADIUS, Color::rgba(30, 30, 30, 20));
        // Pawn
        frame.circle(location.screen_center(), PAWN_RADIUS, color.with_alpha((255.0 - alpha_pulse).max(0.0) as u8));
    }

    /// Returns if translation was successful
    pub fn translate(&mut self, delta_x: i32, delta_y: i32) -> bool {
        let new_pos = ((self.pos.x as i32) + delta_x, (self.pos.y as i32) + delta_y).try_into();

        if let Ok(new_pos) = new_pos {
            self.pos = new_pos;
            true
        }
        else {
            false
        }
    }
}