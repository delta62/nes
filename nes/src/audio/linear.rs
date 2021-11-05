pub struct LinearCounter {
    counter: u8,
    reload_flag: bool,
    reload_val: u8,
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            reload_flag: false,
            reload_val: 0,
        }
    }

    pub fn clock(&mut self, control_flag: bool) {
        if self.reload_flag {
            self.counter = self.reload_val;
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if !control_flag {
            self.set_reload_flag(false);
        }
    }

    pub fn nonzero(&self) -> bool {
        self.counter > 0
    }

    pub fn set_reload(&mut self, reload: u8) {
        self.reload_val = reload;
    }

    pub fn set_reload_flag(&mut self, reload: bool) {
        self.reload_flag = reload;
    }
}
