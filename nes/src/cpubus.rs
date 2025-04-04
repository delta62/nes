use crate::{audio::Apu, input::Input, mapper::PrgMem, mem::Mem, ppu::Ppu, ram::Ram};

pub struct CpuBus {
    pub ram: Ram,
    pub apu: Apu,
    pub ppu: Ppu,
    pub input: Input,
    pub mapper: PrgMem,
}

impl CpuBus {
    pub fn new(ppu: Ppu, input: Input, apu: Apu, mapper: PrgMem, ram: Ram) -> Self {
        Self {
            apu,
            ram,
            ppu,
            input,
            mapper,
        }
    }
}

impl Mem for CpuBus {
    fn peekb(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram.peekb(addr)
        } else if addr < 0x4000 {
            self.ppu.peekb(addr)
        } else if addr < 0x4016 {
            self.apu.peekb(addr)
        } else if addr < 0x4018 {
            self.input.peekb(addr)
        } else if addr < 0x6000 {
            0
        } else {
            self.mapper.as_ref().peekb(addr)
        }
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram.loadb(addr)
        } else if addr < 0x4000 {
            self.ppu.loadb(addr)
        } else if addr < 0x4016 {
            self.apu.loadb(addr)
        } else if addr < 0x4018 {
            self.input.loadb(addr)
        } else if addr < 0x6000 {
            0
        } else {
            self.mapper.as_mut().loadb(addr)
        }
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram.storeb(addr, val)
        } else if addr < 0x4000 {
            self.ppu.storeb(addr, val)
        } else if addr < 0x4016 {
            self.apu.storeb(addr, val);
        } else if addr == 0x4016 {
            self.input.storeb(addr, val)
        } else if addr == 0x4017 {
            self.apu.storeb(addr, val);
        } else if addr < 0x6000 {
            // nop
        } else {
            self.mapper.as_mut().storeb(addr, val);
        }
    }
}
