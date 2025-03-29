use super::{ChrMem, PrgMem};
use crate::{rom::Rom, Mem};

const PRG_ROM_BANK_LEN: usize = 16384; // 16KiB

pub struct Chr(Vec<u8>);

impl Mem for Chr {
    fn peekb(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn storeb(&mut self, _addr: u16, _val: u8) {}
}

pub struct Prog(Vec<u8>);

impl Mem for Prog {
    fn peekb(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            0
        } else if self.0.len() > PRG_ROM_BANK_LEN {
            // This nrom cart has 2 banks of PRG ROM (32KiB)
            self.0[addr as usize & 0x7FFF]
        } else {
            // This nrom cart has 1 bank of PRG ROM (16KiB)
            self.0[addr as usize & 0x3FFF]
        }
    }

    fn storeb(&mut self, _addr: u16, _val: u8) {}
}

pub fn nrom(rom: Rom) -> (ChrMem, PrgMem) {
    let Rom { prg, chr, .. } = rom;
    (ChrMem(Box::new(Chr(chr))), PrgMem(Box::new(Prog(prg))))
}
