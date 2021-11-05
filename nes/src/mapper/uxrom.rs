use crate::rom::Rom;
use log::error;
use super::{Mapper, MapperResult};

const PRG_ROM_BANK_LEN: usize = 0x4000; // 16KiB

pub struct Uxrom {
    bank: usize,
    rom: Box<Rom>,
}

impl Uxrom {
    pub fn new(rom: Box<Rom>) -> Self {
        let bank = 0;
        Self { bank, rom }
    }
}

impl Mapper for Uxrom {
    fn prg_loadb(&mut self, addr: u16) -> u8 {
        let addr = addr as usize;

        if addr >= 0xC000 {
            // fixed to the last bank of prg-rom
            let bank_number = self.rom.header.prg_rom_size - 1;
            let base_addr = bank_number as usize * PRG_ROM_BANK_LEN;
            let prg_addr = base_addr + (addr & 0x3FFF);

            self.rom.prg[prg_addr]
        } else if addr >= 0x8000 {
            let bank_number = self.bank;
            let base_addr = bank_number * PRG_ROM_BANK_LEN;
            let prg_addr = base_addr + (addr & 0x3FFF);

            self.rom.prg[prg_addr]
        } else {
            error!("Load from PRG 0x{:04X}", addr);
            0
        }
    }

    fn prg_storeb(&mut self, _addr: u16, val: u8) {
        self.bank = mask!(val, 0x0F) as usize;
    }

    fn chr_loadb(&mut self, addr: u16) -> u8 {
        self.chr_peekb(addr)
    }

    fn chr_peekb(&self, addr: u16) -> u8 {
        self.rom.prg[addr as usize]
    }

    fn chr_storeb(&mut self, addr: u16, val: u8) {
        self.rom.prg[addr as usize] = val;
    }

    fn next_scanline(&mut self) -> MapperResult {
        MapperResult::Continue
    }
}
