use super::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::game::input::GameInput;
use crate::game::player::{Player, PlayerSide};
use crate::game::tile::{Tile, TilePos};
use crate::game::wall::{Wall, WallOrientation, WallPos};
use crate::raylib::{get_random_value, Color, Frame};
use alloc::format;
use alloc::string::String;
use core::ops::Index;

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

#[derive(Debug)]
pub enum GameSubState {
    PlayerMove(PlayerMove),
    MoveTransition
}

#[derive(Debug)]
pub enum PlayerMove {
    WallPlacement{ location: WallPos, orientation: WallOrientation },
    PlayerMovement(Option<TilePos>),
}

const DEBUG_DRAW_WALL_INFO: bool = false;
const DEBUG_PLAYER_INFO: bool = true;
pub const TILES_DIM: usize = 9;
/// Walls are 2x1, which means you can place them in between two tiles
/// Also it's pointless to put it on the edges, so we forbid that by shrinking the grid
pub const WALL_POINTS_DIM: usize = TILES_DIM - 1;

pub struct Game {
    state: GameState,
    world: World,

    elapsed_time: f32,
    debug_message: String,
}

pub struct World {
    game_mode: GameMode,

    tiles: [Tile; TILES_DIM * TILES_DIM],
    walls: [Option<Wall>; WALL_POINTS_DIM * WALL_POINTS_DIM],

    players: [Player; 2],
    active_player: PlayerSide,

    next_state: Option<GameState>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            state: GameState::Running(
                GameSubState::PlayerMove(PlayerMove::PlayerMovement(None))
            ),
            world: World {
                game_mode: GameMode::Pass,
                tiles: core::array::from_fn(|index| {
                    let x = index % TILES_DIM;
                    let y = index / TILES_DIM;
                    Tile::new(TilePos::new(x, y).unwrap())
                }),
                walls: core::array::from_fn(|index| {
                    if get_random_value(1, 20) < 5 {
                        let x = index % WALL_POINTS_DIM;
                        let y = index / WALL_POINTS_DIM;

                        let is_vertical = get_random_value(1, 10) <= 5;

                        Some(Wall {
                            pos: (x, y).try_into().unwrap(),
                            orientation: if is_vertical { WallOrientation::Vertical } else { WallOrientation::Horizontal },
                        })
                    } else {
                        None
                    }
                }),
                players: [
                    Player::new(PlayerSide::White),
                    Player::new(PlayerSide::Black)
                ],
                active_player: PlayerSide::White,
                next_state: None,
            },

            elapsed_time: 0.0,

            debug_message: String::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Game::new();
    }
}

// ===== Drawing =====
pub const SHADOW_COLOR: Color = Color::rgba(20, 20, 20, 50);


// ===== Logic =====
impl Game {
    pub fn draw(&self, input: &GameInput, frame: &mut Frame) {
        match &self.state {
            GameState::Running(sub_state) => self.draw_running(input, sub_state, frame),
            GameState::Error(info) => self.draw_error(info, frame),
        }
    }

    fn draw_error(&self, error_info: &ErrorInfo, frame: &mut Frame) {
        frame.const_text(c"Error occurred :(", (100, 20).into(), 64, Color::RED);
        frame.text(&error_info.message, (100, 100).into(), 32, Color::WHITE);
        frame.text(&error_info.origination, (100, 150).into(), 24, Color::WHITE);

        frame.const_text(c"Press ZL + Minus to reset.", (100, 400).into(), 24, Color::BLUE)
    }

    fn get_player(&self, player_side: &PlayerSide) -> &Player {
        self.world.players.index(player_side.index())
    }

    fn draw_running(&self, input: &GameInput, sub_state: &GameSubState, frame: &mut Frame) {
        // Tiles
        for tile in self.world.tiles.iter() {
            tile.draw_tile(frame);
        }

        // Players
        for player in self.world.players.iter() {
            player.draw(frame);
        }

        // Wall Shadows
        for wall in self.world.walls.iter().flatten() {
            wall.draw_shadow(frame);
        }

        // Walls
        for wall in self.world.walls.iter().flatten() {
            wall.draw(frame);
        }

        // UI
        match self.world.game_mode {
            GameMode::Pass => {
                let y = SCREEN_HEIGHT - 80;
                frame.const_text(c"White", (10, y).into(), 24, Color::WHITE);
                frame.const_text_right_align(c"Black", 24, SCREEN_WIDTH, 10, y as i32, Color::WHITE);

                Wall::draw_ui_walls(self.get_player(&PlayerSide::White).available_walls, PlayerSide::White, frame);
                Wall::draw_ui_walls(self.get_player(&PlayerSide::Black).available_walls, PlayerSide::Black, frame);
            }
        }

        // PlayerMove specific draw
        match sub_state {
            GameSubState::PlayerMove(player_move) => {
                match player_move {
                    PlayerMove::PlayerMovement(location) => {
                        // Draw a ghost version of the current player
                        if let Some(location) = location {
                            self.get_player(&self.world.active_player).draw_ghost(location, frame, self.elapsed_time);
                        }
                    }
                    PlayerMove::WallPlacement { location, orientation } => {
                        // Draw a ghost version of the new wall
                        Wall::draw_ghost(frame, *location, orientation, self.elapsed_time);
                    }
                }
            }
            GameSubState::MoveTransition => {}
        }

        // =================== Debug ===================
        if DEBUG_DRAW_WALL_INFO {
            for i in 0..WALL_POINTS_DIM {
                for j in 0..WALL_POINTS_DIM {
                    let index = i * WALL_POINTS_DIM + j;

                    let x = (i * 30) + 20;
                    let y = (j * 30) + 20;

                    if let Some(wall) = self.world.walls.get(index).unwrap() {
                        frame.rect(
                            (x, y, 20, 20).into(),
                            Color::WHITE
                        );

                        let text = if let WallOrientation::Vertical = wall.orientation { "V" } else { "H" };

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

        if DEBUG_PLAYER_INFO {
            frame.text(&format!("{:?}", sub_state), (10, 10).into(), 20, Color::WHITE);

            frame.text(&format!("direction: {:?}", input.direction), (10, 30).into(), 20, Color::WHITE);

            frame.text(&format!("Rotate wall: {:?}", input.rotate_wall), (10, 50).into(), 20, Color::WHITE);

            frame.text(&format!("Confirm: {:?}", input.confirm), (10, 70).into(), 20, Color::WHITE);

            frame.text(&format!("Switch: {:?}", input.switch_mode), (10, 90).into(), 20, Color::WHITE);

            frame.text(&self.debug_message, (10, 90).into(), 20, Color::WHITE);
        }
    }

    pub fn update(mut self, input: &GameInput, dt: f32) -> Self {
        // Global state transition check
        if let Some(state) = self.world.next_state {
            self.state = state;
            self.world.next_state = None;
        }

        self.elapsed_time += dt;

        // Reset Logic
        if input.reset {
            self.reset();
            return self;
        }

        let state = &mut self.state;
        let world = &mut self.world;

        match state {
            GameState::Running(sub_state) => {
                sub_state.update(input, world);
            }

            GameState::Error(_) => {}
        }

        self
    }
}

impl GameSubState {
    fn update(
        &mut self,
        input: &GameInput,
        world: &mut World,
    ) {
        match self {
            Self::PlayerMove(player_move) => {
                player_move.update(input, world);
            }

            Self::MoveTransition => {
                // ...
            }
        }
    }
}

impl PlayerMove {
    fn switch(&mut self, world: &mut World) {
        *self = match self {
            PlayerMove::PlayerMovement(_) => {
                let other_player = &world.players[world.active_player.other().index()];

                PlayerMove::WallPlacement {
                    location: WallPos::closest_point(other_player.pos),
                    orientation: WallOrientation::Horizontal,
                }
            }

            PlayerMove::WallPlacement { .. } => {
                PlayerMove::PlayerMovement(None)
            }
        }
    }

    fn update(
        &mut self,
        input: &GameInput,
        world: &mut World,
    ) {
        if input.switch_mode {
            self.switch(world);
        }

        let player = &mut world.players[world.active_player.index()];

        // Inputting a choice
        match self {
            PlayerMove::PlayerMovement(location) => 'block: {
                let Some(direction) = input.direction else {
                    break 'block;
                };

                let new_location = player.pos.translate_dir(direction);

                if let Ok(new_location) = new_location {
                    *location = Some(new_location);
                }
            }

            PlayerMove::WallPlacement {
                location,
                orientation,
            } => 'block: {
                if input.rotate_wall {
                    *orientation = orientation.opposite();
                }

                let Some(direction) = input.direction else {
                    break 'block;
                };

                let new_location = location.translate_dir(direction);

                if let Ok(new_location) = new_location {
                    *location = new_location;
                }
            }
        }

        // Selecting a choice
        if !input.confirm {
            return;
        }

        match self {
            PlayerMove::PlayerMovement(Some(location)) => {
                player.move_to(*location);
            },
            PlayerMove::PlayerMovement(None) => {
                // TODO: Show a notification prompting that you need to select a direction first
            }
            PlayerMove::WallPlacement { location, orientation } => {

            }
        }
    }
}