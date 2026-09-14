use crate::game::input::Direction;
use crate::game::tile::TilePosError::{XOutOfBounds, YOutOfBounds};
use crate::game::world::{SHADOW_COLOR, TILES_DIM};
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::game::wall::{WallPos, WallPosError};
use crate::raylib::{Color, Frame, Vec2};

pub const OFFSET_LEFT: usize = (SCREEN_WIDTH - (TILES_DIM * TILE_SIZE)) / 2;
pub const OFFSET_TOP: usize = (SCREEN_HEIGHT - (TILES_DIM * TILE_SIZE)) / 2;

pub const TILE_SIZE: usize = 64;
pub const INNER_TILE_SIZE: usize = 52;
pub const INNER_TILE_OFFSET: usize = (TILE_SIZE - INNER_TILE_SIZE) / 2;

pub struct Tile {
    pub pos: TilePos
}

impl Tile {
    pub const fn new(pos: TilePos) -> Self {
        Self { pos }
    }

    pub fn draw_tile(&self, frame: &mut Frame) {
        let color = if (self.pos.x + self.pos.y).is_multiple_of(2) {
            Color::rgb(200, 200, 200)
        } else {
            Color::rgb(200, 150, 200)
        };

        // Background fill
        let world_x = self.pos.x * TILE_SIZE + OFFSET_LEFT;
        let world_y = self.pos.y * TILE_SIZE + OFFSET_TOP;

        frame.rect((world_x, world_y, TILE_SIZE, TILE_SIZE).into(), color.darken(50));

        // Smaller tile in the center
        let world_x = self.pos.x * TILE_SIZE + OFFSET_LEFT + INNER_TILE_OFFSET;
        let world_y = self.pos.y * TILE_SIZE + OFFSET_TOP + INNER_TILE_OFFSET;

        // Shadow of the tile
        frame.rect((world_x + 3, world_y + 3, INNER_TILE_SIZE, INNER_TILE_SIZE).into(), SHADOW_COLOR);

        // Tile itself
        frame.rect((world_x, world_y, INNER_TILE_SIZE, INNER_TILE_SIZE).into(), color);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl TilePos {
    pub const fn new(x: usize, y: usize) -> Result<Self, TilePosError> {
        if x >= TILES_DIM { return Err(XOutOfBounds); }
        if y >= TILES_DIM { return Err(YOutOfBounds) }

        Ok(Self { x, y })
    }

    pub const fn array_index(&self) -> usize {
        self.y * TILES_DIM + self.x
    }

    pub fn translate(self, dx: i32, dy: i32) -> Result<Self, TilePosError> {
        (self.x as i32 + dx, self.y as i32 + dy).try_into()
    }

    pub fn translate_dir(self, direction: Direction) -> Result<Self, TilePosError> {
        let (dx, dy) = direction.delta();
        self.translate(dx, dy)
    }

    pub const fn x(self) -> usize {
        self.x
    }

    pub const fn y(self) -> usize {
        self.y
    }

    /// Transforms tile position to screen coordinates (centered inside the tile)
    pub fn screen_center(self) -> Vec2 {
        Vec2::new(
            (self.x as f32 + 0.5) * TILE_SIZE as f32
                + OFFSET_LEFT as f32,

            (self.y as f32 + 0.5) * TILE_SIZE as f32
                + OFFSET_TOP as f32,
        )
    }

    /// Calculates a wall pos (if not out of bounds) that is located at a corner of a tile
    pub fn wall_point(self, corner: Corner) -> Result<WallPos, WallPosError> {
        match corner {
            Corner::TopLeft => {
                (self.x.checked_sub(1).ok_or(WallPosError::XOutOfBounds)?,
                 self.y.checked_sub(1).ok_or(WallPosError::YOutOfBounds)?).try_into()
            }
            Corner::TopRight => {
                (self.x,
                 self.y.checked_sub(1).ok_or(WallPosError::YOutOfBounds)?).try_into()
            },
            Corner::BottomLeft => {
                (self.x.checked_sub(1).ok_or(WallPosError::XOutOfBounds)?,
                 self.y).try_into()
            },
            Corner::BottomRight => {
                (self.x, self.y).try_into()
            },
        }
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
        Self::new(x, y)
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