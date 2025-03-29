use crate::mem::Mem;

pub struct Ram(pub Vec<u8>);

impl Ram {
    pub fn new(byte_size: usize) -> Self {
        Self(vec![0; byte_size])
    }
}

impl Mem for Ram {
    fn peekb(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        self.0[addr as usize] = val
    }
}
