use crate::rom::Rom;

mod nrom;
mod uxrom;

#[derive(PartialEq, Eq)]
pub enum MapperResult {
    Continue,
    // Irq,
}

pub trait Mapper {
    /// Load a byte of memory from PRG ROM
    fn prg_loadb(&mut self, addr: u16) -> u8;

    /// Write a byte of memory to PRG ROM
    fn prg_storeb(&mut self, addr: u16, val: u8);

    /// Load a byte of memory from CHR memory
    fn chr_loadb(&mut self, addr: u16) -> u8;

    /// Debugging only. Read a byte from CHR ROM without making any changes
    fn chr_peekb(&self, addr: u16) -> u8;

    /// Store a byte of memory to CHR memory
    fn chr_storeb(&mut self, addr: u16, val: u8);

    fn next_scanline(&mut self) -> MapperResult;
}

/// Given a ROM, create a mapper that can read & write data with it
pub fn create_mapper(rom: Box<Rom>) -> Box<dyn Mapper> {
    match rom.header.ines_mapper() {
        0 => Box::new(nrom::Nrom::new(rom)),
        2 => Box::new(uxrom::Uxrom::new(rom)),
        x => panic!("Unsupported mapper {}", x),
    }
}
