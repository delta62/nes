use crate::rom::Rom;
use super::{Mapper, MapperResult};

const PRG_ROM_BANK_LEN: usize = 16384; // 16KiB

pub struct Nrom {
    pub rom: Box<Rom>,
}

impl Nrom {
    pub fn new(rom: Box<Rom>) -> Nrom {
        Nrom { rom }
    }
}

impl Mapper for Nrom {
    fn prg_loadb(&mut self, addr: u16) -> u8 {
        if addr < 0x8000 {
            0
        } else if self.rom.prg.len() > PRG_ROM_BANK_LEN {
            self.rom.prg[addr as usize & 0x7FFF]
        } else {
            self.rom.prg[addr as usize & 0x3FFF]
        }
    }

    fn prg_storeb(&mut self, _: u16, _: u8) {}

    fn chr_peekb(&self, addr: u16) -> u8 {
        self.rom.chr[addr as usize]
    }

    fn chr_loadb(&mut self, addr: u16) -> u8 {
        self.chr_peekb(addr)
    }

    fn chr_storeb(&mut self, _: u16, _: u8) {}

    fn next_scanline(&mut self) -> MapperResult {
        MapperResult::Continue
    }
}
