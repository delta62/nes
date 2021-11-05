pub struct Divider {
    current: u8,
    period: u8,
}

impl Divider {
    pub fn new() -> Self {
        Self {
            current: 0,
            period: 0,
        }
    }

    pub fn reload(&mut self) {
        self.current = self.period;
    }

    pub fn set_period(&mut self, val: u8) {
        self.period = val;
    }

    pub fn clock(&mut self) -> bool {
        if self.current == 0 {
            self.current = self.period;
            true
        } else {
            self.current -= 1;
            false
        }
    }
}
