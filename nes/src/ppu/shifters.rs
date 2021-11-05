#[derive(Default)]
pub struct PatternShifter {
    nametable_byte: u8,

    attribute_lo: u16,
    attribute_hi: u16,
    attribute_lo_latch: u8,
    attribute_hi_latch: u8,

    pattern_lo: u16,
    pattern_hi: u16,
    pattern_lo_latch: u8,
    pattern_hi_latch: u8,
}

impl PatternShifter {
    pub fn store_nametable(&mut self, val: u8) {
        self.nametable_byte = val;
    }

    pub fn store_attribute(&mut self, val: u8) {
        self.attribute_lo_latch = if val & 0x01 == 0 { 0x00 } else { 0xFF };
        self.attribute_hi_latch = if val & 0x02 == 0 { 0x00 } else { 0xFF };
    }

    pub fn nametable(&self) -> u8 {
        self.nametable_byte
    }

    pub fn attr_val(&self, fine_x: u8) -> u8 {
        let lo = bitn!(self.attribute_lo, 15 - fine_x) << 0;
        let hi = bitn!(self.attribute_hi, 15 - fine_x) << 1;

        (lo + hi) as u8
    }

    pub fn load_pattern_lo(&mut self, val: u8) {
        self.pattern_lo_latch = val;
    }

    pub fn load_pattern_hi(&mut self, val: u8) {
        self.pattern_hi_latch = val;
    }

    pub fn load_pattern_latches(&mut self) {
        self.pattern_lo |= self.pattern_lo_latch as u16;
        self.pattern_hi |= self.pattern_hi_latch as u16;
        self.attribute_lo |= self.attribute_lo_latch as u16;
        self.attribute_hi |= self.attribute_hi_latch as u16;
    }

    pub fn pattern_val(&self, fine_x: u8) -> u8 {
        let lo = bitn!(self.pattern_lo, 15 - fine_x) << 0;
        let hi = bitn!(self.pattern_hi, 15 - fine_x) << 1;

        (lo + hi) as u8
    }

    pub fn shift(&mut self) {
        self.pattern_lo <<= 1;
        self.pattern_hi <<= 1;

        self.attribute_lo <<= 1;
        self.attribute_hi <<= 1;
    }
}
