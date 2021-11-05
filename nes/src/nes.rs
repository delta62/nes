use crate::audio::Apu;
use crate::cpu::Cpu;
use crate::cpubus::CpuBus;
use crate::input::{Input, InputState};
use crate::mapper::create_mapper;
use crate::ppu::Ppu;
use crate::rom::Rom;
use getset::{Getters};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub struct StepResult {
    pub new_frame: bool,
}

#[derive(Getters)]
pub struct Nes {
    #[getset(get = "pub")]
    apu: Rc<RefCell<Apu>>,
    #[getset(get = "pub")]
    cpu: Cpu<CpuBus>,
    input: Rc<RefCell<Input>>,
    #[getset(get = "pub")]
    ppu: Rc<RefCell<Ppu>>,
}

impl Nes {
    pub fn with_rom(rom: Rom) -> Self {
        let rom = Box::new(rom);
        let mapper = create_mapper(rom);
        let mapper = Rc::new(RefCell::new(mapper));

        let ppu = Ppu::new(mapper.clone());
        let ppu = Rc::new(RefCell::new(ppu));

        let apu = Apu::new();
        let apu = Rc::new(RefCell::new(apu));

        let input = Input::new();
        let input = Rc::new(RefCell::new(input));
        let cpu_bus = CpuBus::new(ppu.clone(), input.clone(), apu.clone(), mapper);

        let mut cpu = Cpu::new(cpu_bus);
        cpu.reset();

        Self { apu, cpu, input, ppu }
    }

    pub fn generate_sound<F>(
        &mut self,
        samples: usize,
        audio_buffer: &mut VecDeque<f32>,
        input: &InputState,
        mut on_frame: F,
    )
    where F: FnMut(&[u8]) {
        for _ in 0..samples {
            let StepResult { new_frame } = self.step(input);

            let apu = self.apu.borrow();
            audio_buffer.push_back(apu.sample());

            if new_frame {
                let ppu = self.ppu.borrow();
                on_frame(ppu.screen().as_ref());
            }
        }
    }

    /// Progress emulation by 1 CPU tick
    pub fn step(&mut self, input: &InputState) -> StepResult {
        self.cpu.step();
        self.input.borrow_mut().set(input);

        let mut apu = self.apu.borrow_mut();
        apu.step();

        let mut ppu = self.ppu.borrow_mut();
        let mut new_frame = false;

        for _ in 0..3 {
            let result = ppu.step();

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
