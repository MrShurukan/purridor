use crate::game::player::PlayerSide;
use crate::game::tile::{TilePos, INNER_TILE_OFFSET, OFFSET_LEFT, OFFSET_TOP, TILE_SIZE};
use crate::game::world::{SHADOW_COLOR, WALL_POINTS_DIM};
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::{Color, Frame, Rect};
use alloc::format;
use alloc::string::String;
use libm::sinf;
use crate::game::util::SinePulser;

const WALL_COLOR: Color = Color::rgb(251, 225, 185);

const UI_WALL_OFFSET: usize = 25;
const UI_WALL_WIDTH: usize = 15;
const UI_WALL_HEIGHT: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallPos {
    pub x: usize,
    pub y: usize,
}

impl WallPos {
    pub fn closest_wall_point(grid_coords: &TilePos) -> WallPos {
        WallPos {
            x: grid_coords.x.clamp(0, WALL_POINTS_DIM - 1),
            y: grid_coords.y.clamp(0, WALL_POINTS_DIM - 1),
        }
    }
}

impl TryFrom<(usize, usize)> for WallPos {
    type Error = String;

    fn try_from((x, y): (usize, usize)) -> Result<Self, Self::Error> {
        if x >= WALL_POINTS_DIM { return Err(format!("Invalid x wall position: {}", x)) }
        if y >= WALL_POINTS_DIM { return Err(format!("Invalid y wall position: {}", y)) }

        Ok(Self { x, y })
    }
}

impl TryFrom<(i32, i32)> for WallPos {
    type Error = String;

    fn try_from((x, y): (i32, i32)) -> Result<Self, Self::Error> {
        let x: usize = x.try_into().map_err(|_| format!("Invalid x wall position: {}", x))?;
        let y: usize = y.try_into().map_err(|_| format!("Invalid y wall position: {}", y))?;

        (x, y).try_into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallOrientation {
    Vertical,
    Horizontal,
}

impl WallOrientation {
    pub const fn rotation(self) -> f32 {
        match self {
            WallOrientation::Vertical => 90.0,
            WallOrientation::Horizontal => 0.0,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            WallOrientation::Vertical => WallOrientation::Horizontal,
            WallOrientation::Horizontal => WallOrientation::Vertical,
        }
    }
}

pub struct Wall {
    pub orientation: WallOrientation,
}


impl Wall {
    // Mind the wall grid! It's different from the regular tile grid
    // The grid point on x and y marks the center of the wall.
    // Wall is 2 tiles wide
    fn construct_rect(location: WallPos, dx: usize, dy: usize) -> Rect {
        // Constructing a 2 tiles wide rect
        let world_x = (location.x + 1) * TILE_SIZE + OFFSET_LEFT;
        let world_y = (location.y + 1) * TILE_SIZE + OFFSET_TOP;

        let short_side = INNER_TILE_OFFSET * 2;
        let long_side = 2 * TILE_SIZE + INNER_TILE_OFFSET * 2;

        (world_x + dx, world_y + dy, long_side, short_side).into()
    }

    pub fn draw(&self, location: WallPos, frame: &mut Frame) {
        let rect = Self::construct_rect(location, 0, 0);
        let rotation = self.orientation.rotation();

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, WALL_COLOR);
    }

    pub fn draw_shadow(&self, location: WallPos, frame: &mut Frame) {
        let rect = Self::construct_rect(location, 3, 3);
        let rotation = self.orientation.rotation();

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, SHADOW_COLOR);
    }

    const GHOST_ALPHA_PULSER: SinePulser = SinePulser::new(20.0, 40.0, 2.0);

    pub fn draw_ghost(frame: &mut Frame, location: WallPos, orientation: &WallOrientation, time: f32) {
        let mut rect = Self::construct_rect(location, 3, 3);
        let rotation = orientation.rotation();

        let alpha_pulse = Self::GHOST_ALPHA_PULSER.pulse(time);

        let shadow_color = SHADOW_COLOR.with_alpha(
            SHADOW_COLOR.a.saturating_sub(alpha_pulse as u8)
        );

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, shadow_color);

        rect.x -= 3.0;
        rect.y -= 3.0;

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation,
                            WALL_COLOR.tint(Color::GREEN, 0.5).with_alpha((255.0 - alpha_pulse).max(0.0) as u8));
    }

    pub fn draw_ui_walls(amount: usize, side: PlayerSide, frame: &mut Frame) {
        let y = SCREEN_HEIGHT - 50;

        match side {
            PlayerSide::White => {
                for i in 0..amount {
                    let x = i * UI_WALL_OFFSET + 10;
                    frame.rect((x, y, UI_WALL_WIDTH, UI_WALL_HEIGHT).into(), WALL_COLOR);
                }
            }
            PlayerSide::Black => {
                for i in 0..amount {
                    let x = SCREEN_WIDTH - (i * UI_WALL_OFFSET) - UI_WALL_WIDTH - 10;
                    frame.rect((x, y, UI_WALL_WIDTH, UI_WALL_HEIGHT).into(), WALL_COLOR);
                }
            }
        }
    }
}