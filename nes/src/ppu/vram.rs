use std::cell::RefCell;
use std::rc::Rc;

use crate::mapper::Mapper;
use crate::mem::Mem;

/// Nametable memory
///
/// Stores the layout of the background
pub struct Vram {
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>,
    pub nametables: [u8; 0x0800], // 2 nametables, 0x400 each
    pub palette: [u8; 0x20],
}

impl Vram {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Vram {
        Vram {
            mapper,
            nametables: [0; 0x800],
            palette: [0; 0x20],
        }
    }
}

impl Mem for Vram {
    fn peekb(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;

        if addr < 0x2000 {
            self.mapper.borrow().chr_peekb(addr)
        } else if addr < 0x3F00 {
            self.nametables[addr as usize & 0x07FF]
        } else if addr < 0x4000 {
            self.palette[addr as usize & 0x1F]
        } else {
            unreachable!()
        }
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        let addr = addr & 0x3FFF;

        if addr < 0x2000 {
            let mut mapper = self.mapper.borrow_mut();
            mapper.chr_storeb(addr, val);
        } else if addr < 0x3F00 {
            let addr = addr & 0x07FF;
            self.nametables[addr as usize] = val;
        } else if addr < 0x4000 {
            let mut addr = addr & 0x1F;
            if addr == 0x10 {
                addr = 0x00; // Mirror sprite background color into universal background color
            }
            self.palette[addr as usize] = val;
        } else {
            unreachable!()
        }
    }
}
