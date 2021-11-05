use crate::mem::Mem;

const STROBE_STATE_A:      u8 = 0;
const STROBE_STATE_B:      u8 = 1;
const STROBE_STATE_SELECT: u8 = 2;
const STROBE_STATE_START:  u8 = 3;
const STROBE_STATE_UP:     u8 = 4;
const STROBE_STATE_DOWN:   u8 = 5;
const STROBE_STATE_LEFT:   u8 = 6;
const STROBE_STATE_RIGHT:  u8 = 7;

#[derive(Debug)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub enum ButtonState {
    Press,
    Release,
}

#[derive(Default)]
pub struct InputState {
    pub gamepad1: GamePadState,
}

struct StrobeState(u8);

impl StrobeState {
    fn get(&self, state: &GamePadState) -> bool {
        match self.0 {
            STROBE_STATE_A      => state.a,
            STROBE_STATE_B      => state.b,
            STROBE_STATE_SELECT => state.select,
            STROBE_STATE_START  => state.start,
            STROBE_STATE_UP     => state.up,
            STROBE_STATE_DOWN   => state.down,
            STROBE_STATE_LEFT   => state.left,
            STROBE_STATE_RIGHT  => state.right,
            _                   => unreachable!(),
        }
    }

    fn next(&mut self) {
        self.0 = (self.0 + 1) & 7;
    }

    fn reset(&mut self) {
        self.0 = STROBE_STATE_A;
    }
}

#[derive(Clone, Default)]
pub struct GamePadState {
    pub left: bool,
    pub down: bool,
    pub up: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
}

impl GamePadState {
    pub fn button_press(&mut self, button: Button, state: ButtonState) {
        let pressed = state == ButtonState::Press;

        match button {
            Button::A      => self.a = pressed,
            Button::B      => self.b = pressed,
            Button::Select => self.select = pressed,
            Button::Start  => self.start = pressed,
            Button::Up     => self.up = pressed,
            Button::Down   => self.down = pressed,
            Button::Left   => self.left = pressed,
            Button::Right  => self.right = pressed,
        }
    }
}

pub struct Input {
    gamepad_0: (GamePadState, StrobeState),
}

impl Input {
    pub fn new() -> Self {
        let strobe = StrobeState(STROBE_STATE_A);
        let gamepad = GamePadState {
            left: false,
            down: false,
            up: false,
            right: false,
            a: false,
            b: false,
            select: false,
            start: false,
        };

        Self {
            gamepad_0: (gamepad, strobe),
        }
    }

    pub fn set(&mut self, input: &InputState) {
        self.gamepad_0.0 = input.gamepad1.clone();
    }
}

impl Mem for Input {
    fn peekb(&self, addr: u16) -> u8 {
        if addr == 0x4016 {
            let (gamepad, strobe) = &self.gamepad_0;
            strobe.get(gamepad) as u8
        } else {
            0
        }
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        if addr == 0x4016 {
            let (gamepad, strobe) = &mut self.gamepad_0;
            let result = strobe.get(gamepad) as u8;
            strobe.next();
            result
        } else {
            0
        }
    }

    fn storeb(&mut self, addr: u16, _: u8) {
        if addr == 0x4016 {
            self.gamepad_0.1.reset();
        }
    }
}
