pub struct Timer {
    has_elapsed: bool,
    period: u16,
    current: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            current: 0,
            period: 0,
            has_elapsed: false,
        }
    }

    pub fn tick(&mut self) {
        if self.current == 0 {
            self.current = self.period;

            if self.period > 0 {
                self.has_elapsed = true;
            }
        } else {
            self.has_elapsed = false;
            self.current -= 1;
        }
    }

    pub fn has_elapsed(&self) -> bool {
        self.has_elapsed
    }

    pub fn get_period(&self) -> u16 {
        self.period
    }

    pub fn set_period(&mut self, period: u16) {
        self.period = period;
    }

    pub fn set_period_lo(&mut self, period: u8) {
        let hi = self.period & 0xFF00;
        let lo = period as u16;

        self.period = lo + hi;
    }

    pub fn set_period_hi(&mut self, period: u8) {
        let hi = (period as u16) << 8;
        let lo = self.period & 0x00FF;

        self.period = lo + hi;
    }
}
