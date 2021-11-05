use getset::Setters;

pub enum SpritePriority {
    AboveBackground,
    BelowBackground,
}

#[derive(Clone, Copy, Debug, Default, Setters)]
pub struct SpriteShift {
    counter: u8,
    latch: u8,
    shift_lo: u8,
    shift_hi: u8,
}

impl SpriteShift {
    pub fn set_x(&mut self, val: u8) {
        self.counter = val;
    }

    pub fn set_attributes(&mut self, val: u8) {
        self.latch = val;
    }

    pub fn set_pattern_lo(&mut self, val: u8) {
        self.shift_lo = val;
    }

    pub fn set_pattern_hi(&mut self, val: u8) {
        self.shift_hi = val;
    }

    pub fn palette(&self) -> u8 {
        self.latch & 0x03
    }

    pub fn priority(&self) -> SpritePriority {
        if bit!(self.latch, 5) {
            SpritePriority::BelowBackground
        } else {
            SpritePriority::AboveBackground
        }
    }

    fn mirror_x(&self) -> bool {
        bit!(self.latch, 6)
    }

    pub fn tick(&mut self) -> u8 {
        if self.counter > 0 {
            self.counter -= 1;
            0
        } else {
            self.shift()
        }
    }

    fn shift(&mut self) -> u8 {
        let lo: u8;
        let hi: u8;

        if self.mirror_x() {
            lo = bitn!(self.shift_lo, 0) << 0;
            hi = bitn!(self.shift_hi, 0) << 1;

            self.shift_lo >>= 1;
            self.shift_hi >>= 1;
        } else {
            lo = bitn!(self.shift_lo, 7) << 0;
            hi = bitn!(self.shift_hi, 7) << 1;

            self.shift_lo <<= 1;
            self.shift_hi <<= 1;
        }

        lo | hi
    }
}
