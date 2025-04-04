use crate::mem::Mem;
use bitflags::bitflags;
use log::{error, warn};

const STROBE_STATE_A: u8 = 0;
const STROBE_STATE_B: u8 = 1;
const STROBE_STATE_SELECT: u8 = 2;
const STROBE_STATE_START: u8 = 3;
const STROBE_STATE_UP: u8 = 4;
const STROBE_STATE_DOWN: u8 = 5;
const STROBE_STATE_LEFT: u8 = 6;
const STROBE_STATE_RIGHT: u8 = 7;

#[derive(Default)]
pub struct InputState {
    pub gamepad1: Option<Buttons>,
    pub gamepad2: Option<Buttons>,
}

bitflags! {
    #[derive(Default, Debug, Copy, Clone)]
    pub struct Buttons: u8 {
        const LEFT = 1 << 0;
        const RIGHT = 1 << 1;
        const UP = 1 << 2;
        const DOWN = 1 << 3;
        const A = 1 << 4;
        const B = 1 << 5;
        const SELECT = 1 << 6;
        const START = 1 << 7;
    }
}

#[derive(Default)]
struct Strobe {
    step: u8,
    buttons: Buttons,
}

impl Strobe {
    fn get(&self) -> u8 {
        match self.step {
            STROBE_STATE_A => self.buttons.intersects(Buttons::A).into(),
            STROBE_STATE_B => self.buttons.intersects(Buttons::B).into(),
            STROBE_STATE_SELECT => self.buttons.intersects(Buttons::SELECT).into(),
            STROBE_STATE_START => self.buttons.intersects(Buttons::START).into(),
            STROBE_STATE_UP => self.buttons.intersects(Buttons::UP).into(),
            STROBE_STATE_DOWN => self.buttons.intersects(Buttons::DOWN).into(),
            STROBE_STATE_LEFT => self.buttons.intersects(Buttons::LEFT).into(),
            STROBE_STATE_RIGHT => self.buttons.intersects(Buttons::RIGHT).into(),
            _ => 0x01,
        }
    }

    fn next(&mut self) {
        if self.step <= STROBE_STATE_RIGHT {
            self.step += 1
        }
    }

    fn reset_step(&mut self) {
        self.step = STROBE_STATE_A;
    }
}

#[derive(Default)]
pub struct Input {
    strobe_flag: bool,
    port1: Strobe,
    port2: Strobe,
}

impl Input {
    pub fn set(&mut self, input: &InputState) {
        self.port1.buttons = input.gamepad1.unwrap_or(Buttons::empty());
        self.port2.buttons = input.gamepad2.unwrap_or(Buttons::empty());
    }

    pub fn step(&mut self) {
        if self.strobe_flag {
            self.port1.reset_step();
            self.port2.reset_step();
        }
    }
}

impl Mem for Input {
    fn peekb(&self, addr: u16) -> u8 {
        match addr {
            0x4016 => self.port1.get(),
            0x4017 => self.port2.get(),
            _ => 0,
        }
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        match addr {
            0x4016 => {
                let ret = self.port1.get();
                self.port1.next();
                ret
            }
            0x4017 => {
                let ret = self.port2.get();
                self.port2.next();
                ret
            }
            n => {
                error!("Input read on address {n}");
                0
            }
        }
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        if addr == 0x4016 {
            self.strobe_flag = bit!(val, 0);
        } else {
            warn!("Unexpected write to input addr {addr:04X}");
        }
    }
}
