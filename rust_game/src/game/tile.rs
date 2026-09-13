use crate::game::world::{SHADOW_COLOR, TILES_DIM};
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::{Color, Frame, Vec2};
use alloc::format;
use alloc::string::String;
use crate::game::tile::TilePosError::{XOutOfBounds, YOutOfBounds};

pub const OFFSET_LEFT: usize = (SCREEN_WIDTH - (TILES_DIM * TILE_SIZE)) / 2;
pub const OFFSET_TOP: usize = (SCREEN_HEIGHT - (TILES_DIM * TILE_SIZE)) / 2;

pub const TILE_SIZE: usize = 64;
pub const INNER_TILE_SIZE: usize = 52;
pub const INNER_TILE_OFFSET: usize = (TILE_SIZE - INNER_TILE_SIZE) / 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePos {
    pub x: usize,
    pub y: usize,
}

impl TilePos {
    /// Transforms tile position to screen coordinates (centered inside the tile)
    pub fn screen_center(self) -> Vec2 {
        Vec2::new(
            (self.x as f32 + 0.5) * TILE_SIZE as f32
                + OFFSET_LEFT as f32,

            (self.y as f32 + 0.5) * TILE_SIZE as f32
                + OFFSET_TOP as f32,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePosError {
    XOutOfBounds,
    YOutOfBounds,
}

impl TryFrom<(usize, usize)> for TilePos {
    type Error = TilePosError;

    fn try_from((x, y): (usize, usize)) -> Result<Self, Self::Error> {
        if x >= TILES_DIM { return Err(XOutOfBounds); }
        if y >= TILES_DIM { return Err(YOutOfBounds) }

        Ok(Self { x, y })
    }
}

impl TryFrom<(i32, i32)> for TilePos {
    type Error = TilePosError;

    fn try_from((x, y): (i32, i32)) -> Result<Self, Self::Error> {
        let x: usize = x.try_into().map_err(|_| XOutOfBounds)?;
        let y: usize = y.try_into().map_err(|_| YOutOfBounds)?;

        (x, y).try_into()
    }
}

pub fn draw_tile(frame: &mut Frame, location: TilePos) {
    let color = if (location.x + location.y).is_multiple_of(2) {
        Color::rgb(200, 200, 200)
    } else {
        Color::rgb(200, 150, 200)
    };

    // Background fill
    let world_x = location.x * TILE_SIZE + OFFSET_LEFT;
    let world_y = location.y * TILE_SIZE + OFFSET_TOP;

    frame.rect((world_x, world_y, TILE_SIZE, TILE_SIZE).into(), color.darken(50));

    // Smaller tile in the center
    let world_x = location.x * TILE_SIZE + OFFSET_LEFT + INNER_TILE_OFFSET;
    let world_y = location.y * TILE_SIZE + OFFSET_TOP + INNER_TILE_OFFSET;

    // Shadow of the tile
    frame.rect((world_x + 3, world_y + 3, INNER_TILE_SIZE, INNER_TILE_SIZE).into(), SHADOW_COLOR);

    // Tile itself
    frame.rect((world_x, world_y, INNER_TILE_SIZE, INNER_TILE_SIZE).into(), color);
}