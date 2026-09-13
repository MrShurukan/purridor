use alloc::format;
use alloc::string::{String, ToString};
use core::ops::Index;
use crate::game::player::{Player, PlayerSide};
use crate::game::strings::{*};
use crate::raylib::{get_random_value, Button, Color, Frame, Gamepad, Rect, TextEdgeAlignment};
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug)]
pub enum GameState {
    // MainMenu,
    Running(GameSubState),
    // GameOver,
    Error(ErrorInfo)
}

#[derive(Debug)]
pub struct ErrorInfo {
    pub message: String,
    pub origination: String
}

macro_rules! error_state {
    ($message:expr) => {
        GameState::Error(
            ErrorInfo { message: $message.to_string(), origination: format!("{}; line: {}", module_path!(), line!()) }
        )
    };
}

pub enum GameMode {
    /// Game mode where you pass the switch to the other player once you move
    Pass,
    // /// Game mode where switch lies on the table between you and the other person controls
    // /// the game upside down
    // Opposition
}

#[derive(Debug, Copy, Clone)]
pub enum GameSubState {
    PlayerMove,
    MoveTransition
}

const DEBUG_DRAW_WALL_INFO: bool = false;
pub const TILES_DIM: usize = 9;
/// Walls are 2x1, which means you can place them in between two tiles
/// Also it's pointless to put it on the edges, so we forbid that by shrinking the grid
pub const WALL_POINTS_DIM: usize = TILES_DIM - 2;

#[derive(Clone)]
struct Wall {
    is_vertical: bool,
}

pub struct Game {
    state: GameState,
    game_mode: GameMode,
    walls: [Option<Wall>; WALL_POINTS_DIM * WALL_POINTS_DIM],

    players: [Player; 2],
    active_player: PlayerSide,

    gamepad: Gamepad,
    just_reset: bool,
}

impl Game {
    pub fn new() -> Self {
        Game {
            state: GameState::Running(GameSubState::PlayerMove),
            game_mode: GameMode::Pass,
            walls: core::array::from_fn(|_| {
                if get_random_value(1, 20) < 5 {

                    let is_vertical = get_random_value(1, 10) <= 5;

                    Some(Wall { is_vertical })
                } else {
                    None
                }
            }),
            players: [
                Player::new(PlayerSide::White),
                Player::new(PlayerSide::Black)
            ],
            active_player: PlayerSide::White,

            gamepad: Gamepad::new(),
            just_reset: true,
        }
    }

    pub fn reset(&mut self) {
        *self = Game::new();
    }
}

// ===== Drawing =====
pub const TILE_SIZE: usize = 64;
pub const OFFSET_LEFT: usize = (SCREEN_WIDTH - (TILES_DIM * TILE_SIZE)) / 2;
pub const OFFSET_TOP: usize = (SCREEN_HEIGHT - (TILES_DIM * TILE_SIZE)) / 2;

const INNER_TILE_SIZE: usize = 52;
const INNER_TILE_OFFSET: usize = (TILE_SIZE - INNER_TILE_SIZE) / 2;

const SHADOW_COLOR: Color = Color::rgba(20, 20, 20, 50);
const WALL_COLOR: Color = Color::rgb(251, 225, 185);

fn draw_tile(frame: &mut Frame, x: usize, y: usize) {
    let color = if (x + y).is_multiple_of(2) {
        Color::rgb(200, 200, 200)
    } else {
        Color::rgb(200, 150, 200)
    };

    // Background fill
    let world_x = x * TILE_SIZE + OFFSET_LEFT;
    let world_y = y * TILE_SIZE + OFFSET_TOP;

    frame.rect((world_x, world_y, TILE_SIZE, TILE_SIZE).into(), color.darken(50));

    // Smaller tile in the center
    let world_x = x * TILE_SIZE + OFFSET_LEFT + INNER_TILE_OFFSET;
    let world_y = y * TILE_SIZE + OFFSET_TOP + INNER_TILE_OFFSET;

    // Shadow of the tile
    frame.rect((world_x + 3, world_y + 3, INNER_TILE_SIZE, INNER_TILE_SIZE).into(), SHADOW_COLOR);

    // Tile itself
    frame.rect((world_x, world_y, INNER_TILE_SIZE, INNER_TILE_SIZE).into(), color);
}

impl Wall {
    // Mind the wall grid! It's different from the regular tile grid
    // The grid point on x and y marks the center of the wall.
    // Wall is 2 tiles wide
    fn construct_rect(&self, x: usize, y: usize, dx: usize, dy: usize) -> Rect {
        // Constructing a 2 tiles wide rect
        let world_x = (x + 1) * TILE_SIZE + OFFSET_LEFT;
        let world_y = (y + 1) * TILE_SIZE + OFFSET_TOP;

        let short_side = INNER_TILE_OFFSET * 2;
        let long_side = 2 * TILE_SIZE + INNER_TILE_OFFSET * 2;

        (world_x + dx, world_y + dy, long_side, short_side).into()
    }

    fn draw(&self, frame: &mut Frame, x: usize, y: usize) {
        let rect = self.construct_rect(x, y, 0, 0);
        let rotation = if self.is_vertical { 90.0 } else { 0.0 };

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, WALL_COLOR);
    }

    fn draw_shadow(&self, frame: &mut Frame, x: usize, y: usize) {
        let rect = self.construct_rect(x, y, 3, 3);
        let rotation = if self.is_vertical { 90.0 } else { 0.0 };

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, SHADOW_COLOR);
    }
}


// ===== Logic =====
impl Game {
    pub fn draw(&self, frame: &mut Frame) {
        match &self.state {
            GameState::Running(sub_state) => self.draw_running(sub_state, frame),
            GameState::Error(info) => self.draw_error(info, frame),
        }
    }

    fn draw_error(&self, error_info: &ErrorInfo, frame: &mut Frame) {
        frame.const_text(c"Error occurred :(", (100, 20).into(), 64, Color::RED);
        frame.text(&error_info.message, (100, 100).into(), 32, Color::WHITE);
        frame.text(&error_info.origination, (100, 150).into(), 24, Color::WHITE);

        frame.const_text(c"Press ZL + Minus to reset.", (100, 400).into(), 24, Color::BLUE)
    }

    fn draw_running(&self, sub_state: &GameSubState, frame: &mut Frame) {
        // Tiles
        for i in 0..TILES_DIM {
            for j in 0..TILES_DIM {
                draw_tile(frame, i, j);
            }
        }

        // Players
        for player in self.players.iter() {
            player.draw(frame);
        }

        // Wall Shadows
        for i in 0..WALL_POINTS_DIM {
            for j in 0..WALL_POINTS_DIM {
                let index = i * WALL_POINTS_DIM + j;

                if let Some(wall) = self.walls.get(index).unwrap() {
                    wall.draw_shadow(frame, i, j);
                }
            }
        }

        // Walls
        for i in 0..WALL_POINTS_DIM {
            for j in 0..WALL_POINTS_DIM {
                let index = i * WALL_POINTS_DIM + j;

                if let Some(wall) = self.walls.get(index).unwrap() {
                    wall.draw(frame, i, j);
                }
            }
        }

        // UI
        match self.game_mode {
            GameMode::Pass => {
                let y = SCREEN_HEIGHT - 80;
                frame.const_text(c"White", (10, y).into(), 24, Color::WHITE);
                frame.const_text_pro(c"Black",24, TextEdgeAlignment::Right, 10, y as i32, Color::WHITE);

                let y = SCREEN_HEIGHT - 50;

                const UI_WALL_OFFSET: usize = 25;
                const UI_WALL_WIDTH: usize = 15;
                const UI_WALL_HEIGHT: usize = 40;

                for i in 0..self.players.index(PlayerSide::White.index()).available_walls {
                    let x = i * UI_WALL_OFFSET + 10;
                    frame.rect((x, y, UI_WALL_WIDTH, UI_WALL_HEIGHT).into(), WALL_COLOR);
                }

                for i in 0..self.players.index(PlayerSide::Black.index()).available_walls {
                    let x = SCREEN_WIDTH - (i * UI_WALL_OFFSET) - UI_WALL_WIDTH - 10;
                    frame.rect((x, y, UI_WALL_WIDTH, UI_WALL_HEIGHT).into(), WALL_COLOR);
                }
            }
        }

        // Debug
        if DEBUG_DRAW_WALL_INFO {
            for i in 0..WALL_POINTS_DIM {
                for j in 0..WALL_POINTS_DIM {
                    let index = i * WALL_POINTS_DIM + j;

                    let x = (i * 30) + 20;
                    let y = (j * 30) + 20;

                    if let Some(wall) = self.walls.get(index).unwrap() {
                        frame.rect(
                            (x, y, 20, 20).into(),
                            Color::WHITE
                        );

                        let text = if wall.is_vertical { "V" } else { "H" };

                        frame.text(text, (x + 3, y).into(), 20, Color::BLACK);
                    }
                    else {
                        frame.rect(
                            (x, y, 20, 20).into(),
                            Color::WHITE.darken(100),
                        );
                    }
                }
            }
        }
    }

    pub fn process(&mut self, dt: f32) {
        // Reset Logic
        if self.gamepad.down(Button::ZL) && self.gamepad.down(Button::Minus) {
            if self.just_reset {
                return
            }

            self.just_reset = true;
            self.reset();
        }
        else {
            self.just_reset = false;
        }

        // Main Logic
        match self.state {
            GameState::Running(mut sub_state) => self.process_running(&mut sub_state, dt),
            GameState::Error(_) => {}
        }
    }

    pub fn process_running(&mut self, sub_state: &mut GameSubState, dt: f32) {

    }
}