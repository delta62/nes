use crate::audio::Apu;
use crate::input::Input;
use crate::mapper::Mapper;
use crate::mem::Mem;
use crate::ppu::Ppu;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Ram {
    pub val: [u8; 0x800],
}

impl Mem for Ram {
    fn peekb(&self, addr: u16) -> u8 {
        self.val[addr as usize & 0x7ff]
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        self.val[addr as usize & 0x7ff] = val
    }
}

pub struct CpuBus {
    pub ram: Ram,
    apu: Rc<RefCell<Apu>>,
    pub ppu: Rc<RefCell<Ppu>>,
    pub input: Rc<RefCell<Input>>,
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>,
}

impl CpuBus {
    pub fn new(
        ppu: Rc<RefCell<Ppu>>,
        input: Rc<RefCell<Input>>,
        apu: Rc<RefCell<Apu>>,
        mapper: Rc<RefCell<Box<dyn Mapper>>>
    ) -> Self {
        Self {
            apu,
            ram: Ram { val: [0; 0x800] },
            ppu,
            input,
            mapper,
        }
    }

    pub fn audio_sample(&self) -> f32 {
        self.apu.borrow().sample()
    }
}

impl Mem for CpuBus {
    fn peekb(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram.peekb(addr)
        } else if addr < 0x4000 {
            self.ppu.borrow().peekb(addr)
        } else if addr == 0x4016 {
            self.input.borrow().peekb(addr)
        } else if addr <= 0x4018 {
            self.apu.borrow().peekb(addr)
        } else if addr < 0x6000 {
            0
        } else {
            let mut mapper = self.mapper.borrow_mut();
            mapper.prg_loadb(addr)
        }
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram.loadb(addr)
        } else if addr < 0x4000 {
            self.ppu.borrow_mut().loadb(addr)
        } else if addr == 0x4016 {
            self.input.borrow_mut().loadb(addr)
        } else if addr <= 0x4018 {
            self.apu.borrow_mut().loadb(addr)
        } else if addr < 0x6000 {
            0
        } else {
            let mut mapper = self.mapper.borrow_mut();
            mapper.prg_loadb(addr)
        }
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram.storeb(addr, val)
        } else if addr < 0x4000 {
            self.ppu.borrow_mut().storeb(addr, val)
        } else if addr == 0x4016 {
            self.input.borrow_mut().storeb(addr, val)
        } else if addr <= 0x4018 {
            self.apu.borrow_mut().storeb(addr, val);
        } else if addr < 0x6000 {
            // nop
        } else {
            let mut mapper = self.mapper.borrow_mut();
            mapper.prg_storeb(addr, val);
        }
    }
}
