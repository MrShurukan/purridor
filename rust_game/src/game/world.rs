use crate::game::player::{Player, PlayerSide};
use crate::raylib::{get_random_value, Button, Color, Frame, Gamepad, Rect};
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub enum GameState {
    MainMenu,
    Running,
    GameOver
}

pub enum GameMode {
    /// Game mode where you pass the switch to the other player once you move
    Pass,
    /// Game mode where switch lies on the table between you and the other person controls
    /// the game upside down
    Opposition
}

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
            state: GameState::Running,
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

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, Color::rgb(251, 225, 185));
    }

    fn draw_shadow(&self, frame: &mut Frame, x: usize, y: usize) {
        let rect = self.construct_rect(x, y, 3, 3);
        let rotation = if self.is_vertical { 90.0 } else { 0.0 };

        frame.rect_rotation(rect, (0.5, 0.5).into(), rotation, SHADOW_COLOR);
    }
}


// ===== Logic =====
const DEBUG_DRAW_WALL_INFO: bool = true;
impl Game {
    pub fn draw(&self, frame: &mut Frame) {
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
        if self.gamepad.down(Button::ZL) && self.gamepad.down(Button::Minus) {
            if self.just_reset {
                return
            }

            self.just_reset = true;
            self.reset()
        }
        else {
            self.just_reset = false;
        }
    }
}