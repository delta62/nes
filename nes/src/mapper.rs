mod nrom;
// mod uxrom;

use crate::{rom::Rom, Mem};
use nrom::nrom;

pub struct PrgMem(pub Box<dyn Mem + Send>);

impl AsRef<dyn Mem + Send> for PrgMem {
    fn as_ref(&self) -> &(dyn Mem + Send + 'static) {
        self.0.as_ref()
    }
}

impl AsMut<dyn Mem + Send> for PrgMem {
    fn as_mut(&mut self) -> &mut (dyn Mem + Send + 'static) {
        self.0.as_mut()
    }
}

pub struct ChrMem(pub Box<dyn Mem + Send>);

impl AsRef<dyn Mem + Send> for ChrMem {
    fn as_ref(&self) -> &(dyn Mem + Send + 'static) {
        self.0.as_ref()
    }
}

impl AsMut<dyn Mem + Send> for ChrMem {
    fn as_mut(&mut self) -> &mut (dyn Mem + Send + 'static) {
        self.0.as_mut()
    }
}

type SplitRom = (ChrMem, PrgMem);

/// Given a ROM, create a mapper that can read & write data with it
pub fn create_mapper(rom: Rom) -> SplitRom {
    match rom.header.ines_mapper() {
        0 => nrom(rom),
        x => panic!("Unsupported mapper {}", x),
    }
}
