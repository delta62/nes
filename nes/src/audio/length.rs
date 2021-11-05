static LENGTH_LOOKUP: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06,
    0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16,
    0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub struct LengthCounter {
    halt: bool,
    length: u8,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            halt: false,
            length: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.halt {
            return;
        }

        if self.length > 0 {
            self.length -= 1;
        }
    }

    pub fn set_zero(&mut self) {
        self.length = 0;
    }

    pub fn get(&self) -> u8 {
        self.length
    }

    pub fn set(&mut self, val: usize) {
        self.length = LENGTH_LOOKUP[val];
    }

    pub fn mute(&self) -> bool {
        self.length == 0
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }
}