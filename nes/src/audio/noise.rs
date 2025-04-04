use super::channel::Channel;
use super::envelope::Envelope;
use super::length::LengthCounter;
use super::timer::Timer;

const LENGTH_LOOKUP: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    envelope: Envelope,
    length: LengthCounter,
    mode_flag: bool,
    shift: NoiseShift,
    timer: Timer,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            envelope: Envelope::new(),
            length: LengthCounter::new(),
            mode_flag: false,
            shift: NoiseShift::new(),
            timer: Timer::new(),
        }
    }

    pub fn set_loop_period(&mut self, val: u8) {
        let mode_flag = bit!(val, 7);
        let idx = mask!(val, 0x0F) as usize;
        let len = LENGTH_LOOKUP[idx];

        self.mode_flag = mode_flag;
        self.timer.set_period(len);
    }

    pub fn set_len(&mut self, val: u8) {
        let load = (mask!(val, 0xF8) >> 3) as usize;

        self.length.set(load);
        self.envelope.restart();
    }

    pub fn set_halt_const_envelope(&mut self, val: u8) {
        let halt = bit!(val, 5);
        let const_vol = bit!(val, 4);
        let period = mask!(val, 0x0F);

        self.length.set_halt(halt);
        self.envelope.set_constant_volume(const_vol);
        self.envelope.set_volume_period(period);
    }
}

impl Channel for Noise {
    fn clock(&mut self) {
        self.timer.tick();

        if self.timer.has_elapsed() {
            self.shift.shift_right(self.mode_flag);
        }
    }

    fn get(&self) -> u8 {
        if self.length.mute() || self.shift.is_zero() {
            0
        } else {
            self.envelope.get()
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length.set_zero();
        }
    }

    fn half_frame_clock(&mut self) {
        self.length.clock();
    }

    fn quarter_frame_clock(&mut self) {
        self.envelope.clock();
    }

    fn is_running(&self) -> bool {
        self.length.get() > 0
    }
}

struct NoiseShift {
    val: u16,
}

impl NoiseShift {
    fn new() -> Self {
        Self { val: 1 }
    }

    fn shift_right(&mut self, bit6: bool) {
        let feedback_bit = if bit6 {
            bitn!(self.val, 6)
        } else {
            bitn!(self.val, 1)
        };
        let feedback = (feedback_bit ^ (bitn!(self.val, 0))) << 14;

        self.val >>= 1;
        self.val |= feedback;
    }

    fn is_zero(&self) -> bool {
        bit!(self.val, 0)
    }
}
