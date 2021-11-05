static WAVEFORM: [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

static DUTY_CYCLES: [u8; 4] = [
    0b01000000, // 12.5%
    0b01100000, // 25%
    0b01111000, // 50%
    0b10011111, // 25%
];

pub struct SquareSequence {
    duty_cycle: usize,
    duty_idx: u8,
}

impl SquareSequence {
    pub fn new() -> Self {
        Self {
            duty_cycle: 0,
            duty_idx: 0,
        }
    }

    pub fn clock(&mut self) {
        self.duty_idx = (self.duty_idx + 1) % 8;
    }

    pub fn get(&self) -> u8 {
        let duty_cycle = DUTY_CYCLES[self.duty_cycle];
        (1 << self.duty_idx) & duty_cycle
    }

    pub fn set_cycle(&mut self, cycle: usize) {
        self.duty_cycle = cycle;
    }

    pub fn reset_idx(&mut self) {
        self.duty_idx = 0;
    }
}

pub struct TriangleSequence {
    cycle_idx: usize,
}

impl TriangleSequence {
    pub fn new() -> Self {
        Self {
            cycle_idx: 0,
        }
    }

    pub fn get(&self) -> u8 {
        WAVEFORM[self.cycle_idx]
    }

    pub fn clock(&mut self) {
        self.cycle_idx = (self.cycle_idx + 1) % 32;
    }
}
