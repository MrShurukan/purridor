use super::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::game::input::{Direction, GameInput};
use crate::game::player::{Player, PlayerSide};
use crate::game::tile::{Corner, Tile, TilePos};
use crate::game::wall::{Wall, WallOrientation, WallPos};
use crate::raylib::{Color, Frame, Texture, TextureError};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

type GS = GameState;

#[derive(Debug)]
pub enum GameState {
    // MainMenu,
    Running(GSS),
    GameOver { victory: PlayerSide },
    Error(ErrorInfo)
}

#[derive(Debug)]
pub struct ErrorInfo {
    pub message: String,
    pub origination: String
}

macro_rules! error_state {
    ($message:expr) => {
        Some(GS::Error(
            ErrorInfo { message: $message, origination: format!("{}; line: {}", module_path!(), line!()) }
        ))
    };
}

type GM = GameMode;

pub enum GameMode {
    /// Game mode where you pass the switch to the other player once you move
    Pass,
    // /// Game mode where switch lies on the table between you and the other person controls
    // /// the game upside down
    // Opposition
}

type GSS = GameSubState;

#[derive(Debug)]
pub enum GameSubState {
    PlayerTurn(PlayerMove),
    MoveTransition
}

type PM = PlayerMove;

#[derive(Debug)]
pub enum PlayerMove {
    WallPlacement{ location: WallPos, orientation: WallOrientation },
    Movement(PMV),
}

type PMV = PlayerMoveVariant;

#[derive(Debug, Clone)]
pub enum PlayerMoveVariant {
    /// Regular move to a selected direction.
    Regular(Direction),
    /// Hop over a player (you can also choose where you land)
    PlayerHop(Direction)
}

const DEBUG_DRAW_WALL_INFO: bool = false;
const DEBUG_PLAYER_INFO: bool = false;
pub const TILES_DIM: usize = 9;
/// Walls are 2x1, which means you can place them in between two tiles
/// Also it's pointless to put it on the edges, so we forbid that by shrinking the grid
pub const WALL_POINTS_DIM: usize = TILES_DIM - 1;

pub struct Assets {
    pub white_avatar: Texture,
    pub black_avatar: Texture,
    pub white_avatar_win: Texture,
    pub black_avatar_win: Texture,
}

impl Assets {
    pub fn load() -> Result<Self, TextureError> {
        Ok(Self {
            white_avatar:
                Texture::load(
                    c"romfs:/textures/koska.png"
                )?,

            black_avatar:
                Texture::load(
                    c"romfs:/textures/svechka.png"
                )?,

            white_avatar_win:
                Texture::load(
                    c"romfs:/textures/koska_win.png"
                )?,

            black_avatar_win:
                Texture::load(
                    c"romfs:/textures/svechka_win.png"
                )?,
        })
    }
}

pub struct Game {
    state: GameState,
    world: World,

    elapsed_time: f32,
    debug_message: String,

    assets: Assets,
}

type WallGrid = [Option<Wall>; WALL_POINTS_DIM * WALL_POINTS_DIM];
type TileGrid = [Tile; TILES_DIM * TILES_DIM];

pub struct World {
    game_mode: GameMode,

    tiles: TileGrid,
    walls: WallGrid,

    players: [Player; 2],
    active_player: PlayerSide,

    next_state: Option<GameState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallPlacementError {
    Collision,
    Overlap,
    PlayerEntrapped(PlayerSide)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegularMoveError {
    WallBlock,
    InvalidTarget,
    PlayerOverlap
}

impl World {
    /// Tries to calculate what player can do if he chooses to move (so that the choice can be valid from
    /// the start).
    ///
    /// It is potentially possible player will not have a move at all. But at most 6 moves is possible
    /// under ideal conditions
    pub fn calculate_possible_movement(&self, player_side: PlayerSide) -> Vec<PlayerMoveVariant> {
        let direction_priority = match player_side {
            PlayerSide::White => [Direction::Up, Direction::Left, Direction::Right, Direction::Down],
            PlayerSide::Black => [Direction::Down, Direction::Left, Direction::Right, Direction::Up],
        };

        // This giant filter chain yields first possible move (or None)
        // But it's giving a priority to a regular move over
        direction_priority.into_iter()
            // A wall mustn't block path
            .filter(|dir|
                !Self::wall_blocks(self.get_player(player_side).pos(), *dir, &self.walls)
            )
            // It should be a move within the board
            .filter_map(|dir|
                self.get_player(player_side).pos().translate_dir(dir)
                    .map(|target| (target, dir)).ok()
            )
            .filter_map(|(target, dir)| {
                // If players don't overlap it's a valid position
                if self.get_player(player_side.other()).pos() != target {
                    return Some(PMV::Regular(dir));
                }

                // If players overlap we need to consider hopping
                let hop = dir.exclude_opposite().into_iter()
                    // A wall mustn't block path
                    .filter(|dir|
                        !Self::wall_blocks(self.get_player(player_side.other()).pos(), *dir, &self.walls)
                    )
                    // It should be a move within the board
                    .filter_map(|dir|
                        self.get_player(player_side.other()).pos().translate_dir(dir)
                            .map(|_target| dir).ok()
                    )
                    .next();

                if let Some(hop) = hop {
                    return Some(PlayerMoveVariant::PlayerHop(hop));
                }

                None
            })
            .collect()
    }

    pub fn regular_move_possible(&self, player_side: PlayerSide, direction: Direction) -> Result<(), RegularMoveError> {
        // Must not be blocked by a wall
        let cur_pos = self.get_player(player_side).pos();
        if World::wall_blocks(cur_pos, direction, &self.walls) {
            return Err(RegularMoveError::WallBlock);
        }

        // Must be a valid target
        let Ok(target) = cur_pos.translate_dir(direction) else {
            return Err(RegularMoveError::InvalidTarget);
        };

        // If we land on the enemy, it's not a valid regular move
        if target == self.get_opposing_player().pos() {
            return Err(RegularMoveError::PlayerOverlap);
        }

        Ok(())
    }

    pub fn place_wall(&mut self, pos: WallPos, orientation: WallOrientation) -> Result<(), WallPlacementError> {
        self.can_place_wall(pos, orientation)?;

        self.walls[pos.array_index()] = Some(Wall { pos, orientation });

        Ok(())
    }

    fn place_wall_or_error(&mut self, pos: WallPos, orientation: WallOrientation) {
        if let Err(err) = self.place_wall(pos, orientation) {
            self.next_state = error_state!(format!("Illegal wall placement: {:?}:{:?} ({:?})", pos, orientation, err));
            return;
        }

        if self.get_active_player_mut().decrement_walls().is_err() {
            self.next_state = error_state!("Attempted wall placement when player is out of walls!".to_string());
            return;
        }

        self.next_state = Some(GameState::Running(GSS::MoveTransition));
    }

    #[inline]
    fn check_wall_overlap(
        &self,
        directions: &[Direction],
        wall_orientation: WallOrientation,
        pos: WallPos,
    ) -> Result<(), WallPlacementError> {
        if directions.into_iter()
            .filter_map(|dir| pos.translate_dir(*dir).ok())
            .filter_map(|pos| self.walls[pos.array_index()])
            .filter(|wall| wall.orientation == wall_orientation)
            .next()
            .is_some() {
            return Err(WallPlacementError::Overlap);
        }

        Ok(())
    }

    pub fn can_place_wall(
        &self,
        pos: WallPos,
        orientation: WallOrientation
    ) -> Result<(), WallPlacementError> {
        let index = pos.array_index();

        // 1) No wall must be present there already
        if self.walls[index].is_some() {
            return Err(WallPlacementError::Collision);
        }

        // 2) Walls must not collide (they are 2x1 after all)
        match orientation {
            WallOrientation::Vertical => {
                self.check_wall_overlap(&[Direction::Up, Direction::Down], WallOrientation::Vertical, pos)?
            },
            WallOrientation::Horizontal => {
                self.check_wall_overlap(&[Direction::Left, Direction::Right], WallOrientation::Horizontal, pos)?
            },
        }

        // 3) Placement mustn't trap any player
        let mut walls_copy = self.walls.clone();
        walls_copy[index] = Some(Wall { pos, orientation });

        for player in self.players.iter() {
            // players must be able to reach the other end of the board
            let goal = player.side().other().start_y();

            if !Self::path_available(&walls_copy, player.pos(), goal) {
                return Err(WallPlacementError::PlayerEntrapped(player.side()))
            }
        }

        Ok(())
    }

    /// Checks if path is available from the tile to a row with a specified y level
    fn path_available(
        walls: &WallGrid,
        from: TilePos,
        to_y: usize
    ) -> bool {
        let mut visited = core::array::repeat(false);

        Self::path_available_internal(&mut visited, walls, from, to_y)
    }

    fn path_available_internal(
        visited: &mut [bool; TILES_DIM * TILES_DIM],
        walls: &WallGrid,
        from: TilePos,
        to_y: usize
    ) -> bool {
        let index = from.array_index();

        // If we already visited a tile return
        if visited[index] { return false; }

        // If we found our goal, path is complete!
        if from.y() == to_y {
            return true;
        }

        // Otherwise keep searching

        // Mark current tile as visited
        visited[index] = true;

        // Visit other tiles (if possible)
        [Direction::Up, Direction::Left, Direction::Down, Direction::Right]
            .into_iter()
            .filter_map(|dir| {
                // Don't traverse walls
                if Self::wall_blocks(from, dir, walls) {
                    None
                }
                else {
                    from.translate_dir(dir).ok()
                }
            })
            .any(|next_pos| Self::path_available_internal(visited, walls, next_pos, to_y))
    }

    fn wall_blocks(
        tile: TilePos,
        direction: Direction,
        walls: &WallGrid
    ) -> bool {
        match direction {
            Direction::Right =>
                Self::check_corner_walls(
                    &[Corner::TopRight, Corner::BottomRight], WallOrientation::Vertical,
                    tile, walls),
            Direction::Up =>
                Self::check_corner_walls(
                    &[Corner::TopRight, Corner::TopLeft], WallOrientation::Horizontal,
                    tile, walls),
            Direction::Left =>
                Self::check_corner_walls(
                    &[Corner::TopLeft, Corner::BottomLeft], WallOrientation::Vertical,
                    tile, walls),
            Direction::Down =>
                Self::check_corner_walls(
                    &[Corner::BottomLeft, Corner::BottomRight], WallOrientation::Horizontal,
                    tile, walls),
        }
    }

    /// Checks specified corners of a tile if they have walls in a specified orientation
    #[inline]
    fn check_corner_walls(
        corners: &[Corner],
        orientation: WallOrientation,
        tile: TilePos,
        walls: &WallGrid,
    ) -> bool {
        corners.into_iter()
            .filter_map(|corner| tile.wall_point(*corner).ok())
            .filter_map(|wall_point| walls[wall_point.array_index()])
            .any(|wall| wall.orientation == orientation)
    }

    fn move_active_player(&mut self, location: TilePos) {
        self.players[self.active_player.index()].move_to(location);

        self.next_state = Some(GameState::Running(GSS::MoveTransition));
    }

    #[inline]
    fn get_player(&self, player_side: PlayerSide) -> &Player {
        self.players.index(player_side.index())
    }

    #[inline]
    fn get_active_player(&self) -> &Player {
        self.players.index(self.active_player.index())
    }

    #[inline]
    fn get_active_player_mut(&mut self) -> &mut Player {
        self.players.index_mut(self.active_player.index())
    }

    #[inline]
    fn get_opposing_player(&self) -> &Player {
        self.players.index(self.active_player.other().index())
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            state: GS::Running(
                GSS::PlayerTurn(PM::Movement(PMV::Regular(Direction::Up)))
            ),
            world: World {
                game_mode: GameMode::Pass,
                tiles: core::array::from_fn(|index| {
                    let x = index % TILES_DIM;
                    let y = index / TILES_DIM;
                    Tile::new(TilePos::new(x, y).unwrap())
                }),
                walls: [None; WALL_POINTS_DIM * WALL_POINTS_DIM],
                players: [
                    Player::new(PlayerSide::White),
                    Player::new(PlayerSide::Black)
                ],
                active_player: PlayerSide::White,
                next_state: None,
            },

            elapsed_time: 0.0,

            debug_message: String::new(),

            assets: Assets::load().unwrap(),
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
    pub fn draw(&self, frame: &mut Frame) {
        match &self.state {
            GS::Running(sub_state) => {
                self.draw_running(frame, false);
                self.draw_player_turn(sub_state, frame);
            },
            GS::GameOver { .. } => self.draw_running(frame, true),
            GS::Error(info) => self.draw_error(info, frame),
        }
    }

    fn draw_error(&self, error_info: &ErrorInfo, frame: &mut Frame) {
        frame.const_text(c"Error occurred :(", (100, 20).into(), 64, Color::RED);
        frame.text(&error_info.message, (100, 100).into(), 32, Color::WHITE);
        frame.text(&error_info.origination, (100, 150).into(), 24, Color::WHITE);

        frame.const_text(c"Press ZL + Minus to reset.", (100, 400).into(), 24, Color::BLUE)
    }

    #[inline]
    fn get_player(&self, player_side: PlayerSide) -> &Player {
        self.world.get_player(player_side)
    }

    #[inline]
    fn get_active_player(&self) -> &Player {
        self.world.get_active_player()
    }

    #[inline]
    fn get_active_player_mut(&mut self) -> &mut Player { self.world.get_active_player_mut() }

    #[inline]
    fn get_opposing_player(&self) -> &Player {
        self.world.get_opposing_player()
    }

    fn draw_running(&self, frame: &mut Frame, victory_mode: bool) {
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
                // Walls
                let y = SCREEN_HEIGHT - 80;
                frame.const_text(c"White", (10, y).into(), 24, Color::WHITE);
                frame.const_text_right_align(c"Black", 24, SCREEN_WIDTH, 10, y as i32, Color::WHITE);

                Wall::draw_ui_walls(self.get_player(PlayerSide::White).available_walls(), PlayerSide::White, frame);
                Wall::draw_ui_walls(self.get_player(PlayerSide::Black).available_walls(), PlayerSide::Black, frame);

                // Turn
                let y = 10;
                let x = 10;
                frame.const_text(self.world.active_player.move_string(), (x, y).into(), 24, Color::WHITE);

                let avatar = match (victory_mode, self.world.active_player) {
                    (false, PlayerSide::White) => &self.assets.white_avatar,
                    (false, PlayerSide::Black) => &self.assets.black_avatar,
                    (true, PlayerSide::White) => &self.assets.white_avatar_win,
                    (true, PlayerSide::Black) => &self.assets.black_avatar_win
                };
                frame.texture(avatar, (x, y + 40).into());

                // Help
                let y = 10;
                frame.const_text_right_align(c"Help:", 24, SCREEN_WIDTH, 10, y, Color::WHITE);
                if !victory_mode {
                    frame.const_text_right_align(c"Move: dpad", 24, SCREEN_WIDTH, 10, y + 40, Color::WHITE);
                    frame.const_text_right_align(c"Switch mode:  L/R", 24, SCREEN_WIDTH, 10, y + 60, Color::WHITE);
                    frame.const_text_right_align(c"Confirm:     A", 24, SCREEN_WIDTH, 10, y + 80, Color::WHITE);
                    frame.const_text_right_align(c"Rotate wall:     B", 24, SCREEN_WIDTH, 10, y + 100, Color::WHITE);
                } else {
                    frame.const_text_right_align(c"Reset game: ZL + Minus", 24, SCREEN_WIDTH, 10, y + 40, Color::WHITE);
                }
            }
        }

        // No need to draw anything else in victory screen
        if victory_mode { return; }
    }

    fn draw_player_turn(&self, sub_state: &GameSubState, frame: &mut Frame) {
        // PlayerTurn specific draw
        match sub_state {
            GSS::PlayerTurn(PM::Movement(PMV::Regular(direction))) => {
                let target = self.get_active_player()
                    .pos().translate_dir(*direction);

                // Draw a ghost version of the current player
                if let Ok(target) = target {
                    self.get_active_player().draw_ghost(target, frame, self.elapsed_time);
                }
            },
            GSS::PlayerTurn(PM::Movement(PMV::PlayerHop(direction))) => {
                let target = self.get_opposing_player()
                    .pos().translate_dir(*direction);

                // Draw a ghost of the target
                if let Ok(target) = target {
                    self.get_active_player().draw_ghost(target, frame, self.elapsed_time);
                }
            },
            GSS::PlayerTurn(PM::WallPlacement { location, orientation }) => {
                // Draw a ghost version of the new wall
                Wall::draw_ghost(
                    frame,
                    *location,
                    *orientation,
                    self.world.can_place_wall(*location, *orientation).is_ok(),
                    self.elapsed_time
                );
            },
            GSS::MoveTransition => {}
        }
    }

    pub fn update(&mut self, input: &GameInput, dt: f32) {
        // Global state transition check
        if let Some(state) = self.world.next_state.take() {
            self.state = state;
        }

        self.elapsed_time += dt;

        // Reset Logic
        if input.reset {
            self.reset();
        }

        let state = &mut self.state;
        let world = &mut self.world;

        match state {
            GS::Running(GSS::PlayerTurn(player_move)) => {
                player_move.update(input, world);
            },

            GS::Running(GSS::MoveTransition) => {
                // Potential animations may be played here

                // Victory Condition
                if world.get_active_player().pos().y() == world.active_player.other().start_y() {
                    *state = GS::GameOver { victory: world.active_player };
                    return;
                }

                world.active_player = world.active_player.other();

                if let Some(possible) = world.calculate_possible_movement(world.active_player).first() {
                    *state = GS::Running(GSS::PlayerTurn(PM::Movement(possible.clone())));
                }
                else {
                    *state = error_state!(format!("Player {:?} has no legal moves", world.active_player)).unwrap();
                }
            },

            GS::GameOver { .. } => {},
            GS::Error(_) => {}
        }
    }
}

impl PlayerMove {
    fn switch(&mut self, world: &mut World) {
        *self = match self {
            PM::Movement(_) => {
                // Forbid the switch if the walls are lacking
                // TODO: Maybe highlight it somehow
                if world.get_active_player().available_walls() == 0 {
                    return;
                }

                let other_player = world.get_opposing_player();
                let location = WallPos::closest_point(other_player.pos());
                let orientation = WallOrientation::Horizontal;

                PM::WallPlacement {
                    location,
                    orientation
                }
            }

            PM::WallPlacement { .. } => {
                if let Some(possible) = world.calculate_possible_movement(world.active_player).first() {
                    PM::Movement(possible.clone())
                }
                // In theory there might be no legal move, in which case we simply forbid the switch
                // TODO: What if the player is out of walls too?
                else {
                    return;
                }
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

        // Checking for player pressing direction keys
        // Movement is complicated as it can change subtypes on the fly, for example
        // select the square occupied by the enemy - and you morph into hop mode
        match self {
            PM::Movement(PMV::Regular(_)) => 'block: {
                let Some(direction) = input.direction else {
                    break 'block;
                };

                match world.regular_move_possible(world.active_player, direction) {
                    Ok(_) => {
                        *self = PM::Movement(PMV::Regular(direction));
                    },
                    // We need to try hopping
                    Err(RegularMoveError::PlayerOverlap) => {
                        let potential_hops: Vec<Direction> =
                            world.calculate_possible_movement(world.active_player)
                                .into_iter()
                                .filter_map(|move_variant| {
                                    if let PMV::PlayerHop(dir) = move_variant {
                                        Some(dir)
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                        // If there are no hops, just void the input
                        if potential_hops.is_empty() {
                            break 'block;
                        }

                        // Try to maintain the direction player is facing now
                        // (i.e. try to jump over the enemy in a straight line)
                        if potential_hops.iter().any(|dir| *dir == direction) {
                            *self = PM::Movement(PMV::PlayerHop(direction));
                        }
                        // Get any that popped up in the search otherwise
                        else {
                            *self = PM::Movement(PMV::PlayerHop(*potential_hops.first().unwrap()));
                        }
                    },
                    // Void the input otherwise
                    Err(_) => {}
                }
            },

            // All invalid input here try to fall back to regular move in that direction
            // Otherwise the input is voided
            PM::Movement(PMV::PlayerHop(prev_dir)) => 'block: {
                let Some(direction) = input.direction else {
                    break 'block;
                };

                // If the direction matches what we already had then player wants to switch back
                // to regular movement
                if *prev_dir == direction {
                    if world.regular_move_possible(world.active_player, direction).is_ok() {
                        *self = PM::Movement(PMV::Regular(direction));
                    }
                    break 'block;
                }

                // Calculating from the other player position
                let other_pos = world.get_opposing_player().pos();
                // Must not be blocked by a wall
                if World::wall_blocks(other_pos, direction, &world.walls) {
                    if world.regular_move_possible(world.active_player, direction).is_ok() {
                        *self = PM::Movement(PMV::Regular(direction));
                    }
                    break 'block;
                }

                // Must be a valid target
                let Ok(target) = other_pos.translate_dir(direction) else {
                    if world.regular_move_possible(world.active_player, direction).is_ok() {
                        *self = PM::Movement(PMV::Regular(direction));
                    }
                    break 'block;
                };

                // We can't jump back to ourselves
                if target == world.get_active_player().pos() {
                    if world.regular_move_possible(world.active_player, direction).is_ok() {
                        *self = PM::Movement(PMV::Regular(direction));
                    }
                }
                // Otherwise, it's a valid hop
                else {
                    *self = PM::Movement(PMV::PlayerHop(direction));
                }
            },

            PM::WallPlacement {
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
            PM::Movement(PMV::Regular(dir)) => {
                let Ok(target) = world.get_active_player().pos().translate_dir(*dir) else {
                    world.next_state = error_state!("Out of bounds regular move attempted".to_string());
                    return;
                };

                world.move_active_player(target);
            },
            PM::Movement(PMV::PlayerHop(dir)) => {
                let Ok(target) = world.get_opposing_player().pos().translate_dir(*dir) else {
                    world.next_state = error_state!("Out of bounds hop attempted".to_string());
                    return;
                };

                world.move_active_player(target);
            },
            PM::WallPlacement { location, orientation, .. } => {
                if world.can_place_wall(*location, *orientation).is_ok() {
                    world.place_wall_or_error(*location, *orientation);
                }
            }
        }
    }
}