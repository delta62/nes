struct Divider {
    current: u8,
    period: u8,
}

impl Divider {
    fn new() -> Self {
        Self {
            current: 0,
            period: 0,
        }
    }

    fn clock(&mut self) -> bool {
       if self.current == 0 {
           self.current = self.period;
           true
       } else {
           self.current -= 1;
           false
       }
    }

    fn set_period(&mut self, period: u8) {
        self.period = period;
    }
}

pub struct Envelope {
    const_vol_flag: bool,
    decay_counter: u8,
    divider: Divider,
    loop_flag: bool,
    start_flag: bool,
    volume_period: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            const_vol_flag: false,
            decay_counter: 0,
            divider: Divider::new(),
            loop_flag: false,
            start_flag: false,
            volume_period: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_counter = 15;
            self.divider.set_period(self.volume_period);
        } else {
            let ret = self.divider.clock();

            if ret {
                self.divider.set_period(self.volume_period);

                if self.decay_counter > 0 {
                    self.decay_counter -= 1;
                } else if self.loop_flag {
                    self.decay_counter = 15;
                }
            }
        }
    }

    pub fn restart(&mut self) {
        self.start_flag = true;
    }

    pub fn set_loop(&mut self, loop_flag: bool) {
        self.loop_flag = loop_flag;
    }

    pub fn set_constant_volume(&mut self, constant_volume: bool) {
        self.const_vol_flag = constant_volume;
    }

    pub fn set_volume_period(&mut self, volume_period: u8) {
        self.volume_period = volume_period;
    }

    pub fn get(&self) -> u8 {
        if self.const_vol_flag {
            self.volume_period
        } else {
            self.decay_counter
        }
    }
}
