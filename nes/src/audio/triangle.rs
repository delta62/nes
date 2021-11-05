use super::channel::Channel;
use super::length::LengthCounter;
use super::linear::LinearCounter;
use super::sequencer::TriangleSequence;
use super::timer::Timer;

pub struct Triangle {
    control_flag: bool,
    length: LengthCounter,
    linear: LinearCounter,
    sequencer: TriangleSequence,
    timer: Timer,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            control_flag: false,
            length: LengthCounter::new(),
            linear: LinearCounter::new(),
            sequencer: TriangleSequence::new(),
            timer: Timer::new(),
        }
    }

    pub fn set_control_counter(&mut self, val: u8) {
        let bit7 = bit!(val, 7);

        self.control_flag = bit7;
        self.length.set_halt(bit7);
        self.linear.set_reload(mask!(val, 0x7F));
    }

    pub fn set_timer_lo(&mut self, val: u8) {
        self.timer.set_period_lo(val);
    }

    pub fn set_length_counter_timer_hi(&mut self, val: u8) {
        let length_val = (mask!(val, 0xF8) >> 3) as usize;

        self.length.set(length_val);
        self.timer.set_period_hi(mask!(val, 0x07));
        self.linear.set_reload_flag(true);
    }
}

impl Channel for Triangle {
    fn clock(&mut self) {
        self.timer.tick();

        let timer_elapsed = self.timer.has_elapsed();
        let linear_nonzero = self.linear.nonzero();
        let length_mute = self.length.mute();

        if timer_elapsed && linear_nonzero && !length_mute {
            self.sequencer.clock();
        }
    }

    fn get(&self) -> u8 {
        self.sequencer.get()
    }

    fn quarter_frame_clock(&mut self) {
        self.linear.clock(self.control_flag);
    }

    fn half_frame_clock(&mut self) {
        self.length.clock();
    }

    fn is_running(&self) -> bool {
        self.length.get() > 0
    }

    fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length.mute();
        }
    }
}
