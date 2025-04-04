use super::{ChrMem, PrgMem};
use crate::{Mem, Rom};
use log::error;

const PRG_ROM_BANK_LEN: usize = 0x4000; // 16KiB

struct Chr(Vec<u8>);

impl Mem for Chr {
    fn peekb(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        let addr = (addr as usize) & 0x1FFF;
        self.0[addr] = val;
    }
}

struct Prg {
    num_banks: usize,
    bank: usize,
    bytes: Vec<u8>,
}

impl Prg {
    fn new(bytes: Vec<u8>, num_banks: usize) -> Self {
        let bank = 0;
        Self {
            bytes,
            bank,
            num_banks,
        }
    }
}

impl Mem for Prg {
    fn peekb(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            error!("Load from PRG 0x{:04X}", addr);
            return 0;
        }

        let addr = addr as usize;
        let bank_number = if addr >= 0xC000 {
            // Read from mapped bank
            self.num_banks - 1
        } else {
            // Always reads from last bank
            self.bank
        };

        let base_addr = bank_number * PRG_ROM_BANK_LEN;
        let prg_addr = base_addr + (addr & 0x3FFF);

        self.bytes[prg_addr]
    }

    fn storeb(&mut self, _addr: u16, val: u8) {
        self.bank = (val & 0b0000_0111) as usize;
    }
}

pub fn uxrom(rom: Rom) -> (ChrMem, PrgMem) {
    let num_banks = rom.header.prg_banks();
    let Rom { prg, .. } = rom;
    let chr = vec![0; 0x2000];
    let chr = Box::new(Chr(chr));
    let prg = Box::new(Prg::new(prg, num_banks));
    (ChrMem(chr), PrgMem(prg))
}
