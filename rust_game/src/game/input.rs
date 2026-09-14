use crate::raylib::{Button, Gamepad};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Left,
    Down
}

impl Direction {
    fn from_gamepad(gamepad: &Gamepad) -> Option<Direction> {
        if gamepad.pressed(Button::Up)          { Some(Direction::Up) }
        else if gamepad.pressed(Button::Right)  { Some(Direction::Right) }
        else if gamepad.pressed(Button::Left)   { Some(Direction::Left) }
        else if gamepad.pressed(Button::Down)   { Some(Direction::Down) }
        else                                    { None }
    }

    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::Up    => (0, -1),
            Self::Right => (1, 0),
            Self::Down  => (0, 1),
            Self::Left  => (-1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GameInput {
    pub direction: Option<Direction>,
    pub reset: bool,
    pub switch_mode: bool,
    pub rotate_wall: bool,
    pub confirm: bool,
}

impl GameInput {
    pub fn read(gamepad: &Gamepad) -> Self {
        Self {
            direction: Direction::from_gamepad(gamepad),

            reset:
                (gamepad.pressed(Button::Minus) && gamepad.down(Button::ZL)) ||
                (gamepad.pressed(Button::ZL)    && gamepad.down(Button::Minus)),

            switch_mode:
                gamepad.pressed(Button::L) ||
                gamepad.pressed(Button::R),

            rotate_wall:
                gamepad.pressed(Button::B),

            confirm:
                gamepad.pressed(Button::A),
        }
    }
}