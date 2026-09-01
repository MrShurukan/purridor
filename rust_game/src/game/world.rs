use crate::game::player::Player;
use crate::raylib::{Color, Frame, Rect};
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub enum GameState {
    MainMenu,
    Running,
    GameOver
}

pub enum Tile {
    Pavement { has_mail_box: bool },
    Road,
    BicycleLane,
}

pub const TILES_X: usize = 13;
pub const TILES_Y: usize = 11;

pub struct Game {
    state: GameState,
    tiles: [Tile; TILES_X * TILES_Y],

    player: Player,
    temp_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        Game {
            state: GameState::Running,
            tiles: core::array::from_fn(|index| {
                let r = index / TILES_Y;
                let c = index % TILES_X;

                if index.is_multiple_of(2) {
                    Tile::Road
                } else {
                    Tile::BicycleLane
                }
            }),

            player: Player::new(),
            temp_timer: 0.0,
        }
    }
}

// ===== Drawing =====
pub const TILE_SIZE: usize = 64;
pub const OFFSET_LEFT: usize = (SCREEN_WIDTH - (TILES_X * TILE_SIZE)) / 2;
pub const OFFSET_TOP: usize = SCREEN_HEIGHT - (TILES_Y * TILE_SIZE);

impl Tile {
    fn draw(&self, frame: &mut Frame, x: usize, y: usize) {
        let color = match self {
            Tile::Pavement { .. } => Color::rgb(200, 200, 200),
            Tile::Road => Color::rgb(40, 40, 40),
            Tile::BicycleLane => Color::rgb(200, 150, 150),
        };

        frame.rect((x, y, TILE_SIZE, TILE_SIZE).into(), color)
    }
}

impl Game {
    pub fn draw(&self, frame: &mut Frame) {
        for x in 0..TILES_X {
            for y in 0..TILES_Y {
                let index = x + y * TILES_X;

                let x = x * TILE_SIZE + OFFSET_LEFT;
                let y = y * TILE_SIZE + OFFSET_TOP;

                self.tiles[index].draw(frame, x, y);
            }
        }

        self.player.draw(frame);
    }

    pub fn process(&mut self, dt: f32) {
        self.temp_timer += dt;

        if self.temp_timer > 2.0 {
            self.temp_timer = 0.0;
            self.player.translate(0, -1)
        }
    }
}