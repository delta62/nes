use super::channel::Channel;
use super::timer::Timer;

const RATE_LOOKUP: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214,
    190, 160, 142, 128, 106,  84,  72,  54,
];

pub struct Dmc {
    irq_enabled: bool,
    loop_enabled: bool,
    rate: u16,
    sample_addr: u16,
    sample_len: u16,
    timer: Timer,
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            irq_enabled: false,
            loop_enabled: false,
            rate: 0,
            sample_addr: 0,
            sample_len: 0,
            timer: Timer::new(),
        }
    }

    pub fn set_irq_loop_freq(&mut self, val: u8) {
        self.irq_enabled = bit!(val, 7);
        self.loop_enabled = bit!(val, 6);

        let rate_idx = mask!(val, 0x0F) as usize;
        self.rate = RATE_LOOKUP[rate_idx];
    }

    pub fn set_counter(&mut self, _val: u8) {

    }

    pub fn set_sample_address(&mut self, val: u8) {
        let mut val = val as u16;

        val <<= 6;
        val |= 0xC000;

        self.sample_addr = val;
    }

    pub fn set_sample_length(&mut self, val: u8) {
        self.sample_len = val as u16 * 16 + 1
    }
}

impl Channel for Dmc {
    fn clock(&mut self) {
        self.timer.tick();
    }

    fn set_enabled(&mut self, _enabled: bool) {

    }

    fn get(&self) -> u8 {
        0
    }

    fn is_running(&self) -> bool {
        false
    }

    fn half_frame_clock(&mut self) { }
    fn quarter_frame_clock(&mut self) { }
}
