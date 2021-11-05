pub trait Mem {
    /// For debugging only - show the current value of a byte without modifying
    /// the system state. The real hardware cannot always do this (e.g. reading
    /// from PPUSTATUS), so it should not be used in normal operations.
    fn peekb(&self, addr: u16) -> u8;

    /// For debugging only - show the current value of a word without modifying
    /// the system state. The real hardware cannot always do this (e.g. reading
    /// from PPUSTATUS), so it should not be used in normal operations.
    fn peekw(&self, addr: u16) -> u16 {
        word!(self.peekb(addr), self.peekb(addr + 1))
    }

    /// Load a byte from memory. Depending on what device you are reading from,
    /// this may have side effects. See the component's docs for more
    /// information.
    fn loadb(&mut self, addr: u16) -> u8 {
        self.peekb(addr)
    }

    /// Store a byte into memory
    fn storeb(&mut self, addr: u16, val: u8);

    /// Load a word (sixteen bits) from memory. Memory is stored little endian,
    /// so the low 8 bits of the word will be read from the address and the
    /// high 8 bits of the word will be read from address + 1.
    fn loadw(&mut self, addr: u16) -> u16 {
        word!(self.loadb(addr), self.loadb(addr + 1))
    }

    /// Store a word into memory
    fn storew(&mut self, addr: u16, val: u16) {
        let lo = (val & 0xFF) as u8;
        let hi = ((val >> 8) & 0xFF) as u8;

        self.storeb(addr, lo);
        self.storeb(addr + 1, hi);
    }
}
