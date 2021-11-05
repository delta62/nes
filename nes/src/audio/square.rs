use super::channel::Channel;
use super::envelope::Envelope;
use super::length::LengthCounter;
use super::sequencer::SquareSequence;
use super::sweep::Sweep;
use super::timer::Timer;

pub struct Square {
    envelope: Envelope,
    length: LengthCounter,
    sequencer: SquareSequence,
    sweep: Sweep,
    timer: Timer,
}

impl Square {
    pub fn new(square_idx: u8) -> Self {
        Self {
            envelope: Envelope::new(),
            length: LengthCounter::new(),
            sequencer: SquareSequence::new(),
            sweep: Sweep::new(square_idx == 0),
            timer: Timer::new(),
        }
    }

    pub fn set_duty_length_envelope_divider(&mut self, val: u8) {
        // Bits 6 & 7 are the duty cycle; 0-3
        self.sequencer.set_cycle((mask!(val, 0xC0) >> 6) as usize);

        // Bit 5 is the length counter halt flag
        let bit5 = bit!(val, 5);
        self.envelope.set_loop(bit5);
        self.length.set_halt(bit5);

        // Bit 4 is the constant volume / envelope flag
        self.envelope.set_constant_volume(bit!(val, 4));

        // Bits 0-3 are the volume / envelope divider
        self.envelope.set_volume_period(mask!(val, 0x0F));
    }

    pub fn set_sweep(&mut self, val: u8) {
        self.sweep.update(val);
    }

    pub fn set_timer_lo(&mut self, val: u8) {
        self.timer.set_period_lo(val);
    }

    pub fn set_len_timer_hi(&mut self, val: u8) {
        // The low 3 bits are the timer hi
        self.timer.set_period_hi(mask!(val, 0x07));

        // The high 5 bits are the length counter
        let length = (mask!(val, 0xF8) >> 3) as usize;
        self.length.set(length);

        self.sequencer.reset_idx();
        self.envelope.restart();
    }
}

impl Channel for Square {
    fn clock(&mut self) {
        self.timer.tick();

        if self.timer.has_elapsed() {
            self.sequencer.clock();
        }
    }

    fn get(&self) -> u8 {
        let duty_value = self.sequencer.get();
        let timer_period = self.timer.get_period();

        let sweep_mute = self.sweep.mute(timer_period);
        let timer_mute = timer_period < 8;
        let duty_mute = duty_value == 0;
        let length_mute = self.length.mute();

        if sweep_mute || timer_mute || length_mute || duty_mute {
            0
        } else {
            self.envelope.get()
        }
    }

    fn half_frame_clock(&mut self) {
        self.length.clock();

        let period = self.timer.get_period();
        let target_period = self.sweep.clock(period);

        if let Some(period) = target_period {
            self.timer.set_period(period);
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length.set_zero();
        }
    }

    fn is_running(&self) -> bool {
        self.length.get() > 0
    }

    fn quarter_frame_clock(&mut self) {
        self.envelope.clock();
    }
}
