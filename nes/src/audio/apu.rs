use crate::mem::Mem;
use log::warn;
use super::channel::Channel;
use super::dmc::Dmc;
use super::frame_counter::FrameCounter;
use super::mixer::Mixer;
use super::noise::Noise;
use super::square::Square;
use super::triangle::Triangle;

pub struct ApuState {
    pub square1: u8,
    pub square2: u8,
    pub triangle: u8,
    pub noise: u8,
    pub dmc: u8,
}

pub struct ApuResult {
    pub irq: bool,
}

pub struct Apu {
    dmc: Dmc,
    even_cycle: bool,
    frame_counter: FrameCounter,
    noise: Noise,
    square1: Square,
    square2: Square,
    triangle: Triangle,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            dmc: Dmc::new(),
            even_cycle: true,
            frame_counter: FrameCounter::new(),
            noise: Noise::new(),
            square1: Square::new(1),
            square2: Square::new(2),
            triangle: Triangle::new(),
        }
    }

    pub fn step(&mut self) -> ApuResult {
        self.even_cycle = !self.even_cycle;

        let sequencer_result = self.frame_counter.step();

        if sequencer_result.half_frame {
            self.square1.half_frame_clock();
            self.square2.half_frame_clock();
            self.noise.half_frame_clock();
            self.triangle.half_frame_clock();
        }

        if sequencer_result.quarter_frame {
            self.square1.quarter_frame_clock();
            self.square2.quarter_frame_clock();
            self.noise.quarter_frame_clock();
            self.triangle.quarter_frame_clock();
        }

        if self.even_cycle {
            self.square1.clock();
            self.square2.clock();
            self.noise.clock();
            self.dmc.clock();
        }

        self.triangle.clock();

        ApuResult {
            irq: sequencer_result.irq,
        }
    }

    pub fn sample(&self) -> f32 {
        let state = ApuState {
            square1: self.square1.get(),
            square2: self.square2.get(),
            triangle: self.triangle.get(),
            noise: self.noise.get(),
            dmc: self.dmc.get(),
        };

        Mixer::sample(state)
    }

    fn update_flags(&mut self, val: u8) {
        self.square1.set_enabled(bit!(val, 0));
        self.square2.set_enabled(bit!(val, 1));
        self.triangle.set_enabled(bit!(val, 2));
        self.noise.set_enabled(bit!(val, 3));
        self.dmc.set_enabled(bit!(val, 4));
    }
}

impl Mem for Apu {
    fn peekb(&self, addr: u16) -> u8 {
        if addr != 0x4015 {
            warn!("APU read from {}", addr);
            return 0;
        }

        let sq1 =   if self.square1.is_running()  { 0x01 } else { 0 };
        let sq2 =   if self.square2.is_running()  { 0x02 } else { 0 };
        let tri =   if self.triangle.is_running() { 0x04 } else { 0 };
        let noise = if self.noise.is_running()    { 0x08 } else { 0 };
        let dmc =   if self.dmc.is_running()      { 0x10 } else { 0 };

        dmc | noise | tri | sq2 | sq1
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        if addr != 0x4015 {
            return 0;
        }

        self.frame_counter.reset_irq();

        self.peekb(addr)
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        match addr {
            // Square 1
            0x4000 => self.square1.set_duty_length_envelope_divider(val),
            0x4001 => self.square1.set_sweep(val),
            0x4002 => self.square1.set_timer_lo(val),
            0x4003 => self.square1.set_len_timer_hi(val),
            // Square 2
            0x4004 => self.square2.set_duty_length_envelope_divider(val),
            0x4005 => self.square2.set_sweep(val),
            0x4006 => self.square2.set_timer_lo(val),
            0x4007 => self.square2.set_len_timer_hi(val),
            // Triangle
            0x4008 => self.triangle.set_control_counter(val),
            0x4009 => {} // Unused
            0x400A => self.triangle.set_timer_lo(val),
            0x400B => self.triangle.set_length_counter_timer_hi(val),
            // Noise
            0x400C => self.noise.set_halt_const_envelope(val),
            0x400D => {} // Unused
            0x400E => self.noise.set_loop_period(val),
            0x400F => self.noise.set_len(val),
            // DMC
            0x4010 => self.dmc.set_irq_loop_freq(val),
            0x4011 => self.dmc.set_counter(val),
            0x4012 => self.dmc.set_sample_address(val),
            0x4013 => self.dmc.set_sample_length(val),
            // Flags
            0x4015 => self.update_flags(val),
            // Frame counter
            0x4017 => self.frame_counter.update(val),
            _      => unreachable!(),
        }
    }
}
