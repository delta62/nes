use crate::audio::Apu;
use crate::cpu::Cpu;
use crate::cpubus::CpuBus;
use crate::frame_buffer::Frame;
use crate::input::{Input, InputState};
use crate::mapper::create_mapper;
use crate::ppu::Ppu;
use crate::ram::Ram;
use crate::rom::Rom;

const WRAM_BYTE_SIZE: usize = 0x0800;

pub struct StepResult {
    pub new_frame: bool,
}

pub struct Nes {
    pub cpu: Cpu<CpuBus>,
}

impl Nes {
    pub fn with_rom(rom: Rom) -> Self {
        let (chr, prg) = create_mapper(rom);
        let frame_buffer = Frame::new();
        let ppu = Ppu::new(chr, frame_buffer);
        let apu = Apu::new();
        let ram = Ram::new(WRAM_BYTE_SIZE);
        let input = Input::default();
        let cpu_bus = CpuBus::new(ppu, input, apu, prg, ram);

        let mut cpu = Cpu::new(cpu_bus);
        cpu.reset();

        Self { cpu }
    }

    /// Progress emulation by 1 CPU tick
    pub fn step(&mut self, input: &InputState) -> StepResult {
        self.cpu.step();
        self.cpu.mem.input.set(input);
        self.cpu.mem.apu.step();
        self.cpu.mem.input.step();

        let mut new_frame = false;

        for _ in 0..3 {
            let result = self.cpu.mem.ppu.step();

            if result.vblank_nmi {
                self.cpu.nmi();
            } else if result.scanline_irq {
                self.cpu.irq();
            }

            if result.new_frame {
                new_frame = true;
            }
        }

        StepResult { new_frame }
    }
}
