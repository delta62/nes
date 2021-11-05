use crate::mem::Mem;

#[derive(Debug)]
enum TickState {
    ReadYPosition,
    WriteYPosition(u8),
    ReadTileIndex,
    WriteTileIndex(u8),
    ReadAttributes,
    WriteAttributes(u8),
    ReadXPosition,
    WriteXPosition(u8),
    IncrementN,
    OverflowDetection(usize),
    Phase3ExtraRead(u8),
    BusyLoop,
}

pub struct Oam {
    addr: usize,
    oam: [u8; 0x0100],
    oam2: [u8; 32],
    oam2_index: usize,
    n: usize,
    tick_state: TickState,
}

impl Oam {
    pub fn new() -> Self {
        Self {
            addr: 0,
            oam: [0; 0x0100],
            oam2: [0; 32],
            oam2_index: 0,
            n: 0,
            tick_state: TickState::ReadYPosition,
        }
    }

    pub fn sprite_y(&self, sprite_idx: usize) -> u8 {
        self.oam2[sprite_idx * 4]
    }

    pub fn sprite_addr(&self, sprite_idx: usize) -> u8 {
        self.oam2[sprite_idx * 4 + 1]
    }

    pub fn sprite_attr(&self, sprite_idx: usize) -> u8 {
        self.oam2[sprite_idx * 4 + 2]
    }

    pub fn sprite_x(&self, sprite_idx: usize) -> u8 {
        self.oam2[sprite_idx * 4 + 3]
    }

    pub fn reset_addr(&mut self) {
        self.addr = 0;
    }

    pub fn set_addr(&mut self, addr: u8) {
        self.addr = addr as usize;
    }

    pub fn increment_addr(&mut self) {
        self.addr = (self.addr + 1) % 0x100;
    }

    pub fn addr(&self) -> u16 {
        self.addr as u16
    }

    pub fn reset_oam2(&mut self) {
        self.oam2.fill(0xFF);
        self.oam2_index = 0;
        self.tick_state = TickState::ReadYPosition;
        self.n = 0;
    }

    fn in_range(current_scanline: u16, value: u8) -> bool {
        let value = value as u16;
        current_scanline >= value && current_scanline < value + 8
    }

    pub fn sprite_eval(&mut self, current_scanline: u16) -> bool {
        // return true if sprite overflow is detected
        let mut ret = false;

        match self.tick_state {
            TickState::ReadYPosition => {
                let addr = self.addr + self.n * 4;
                let y_coord = self.oam[addr];
                self.tick_state = TickState::WriteYPosition(y_coord);
            }
            TickState::WriteYPosition(y_pos) => {
                if self.oam2_index == 8 {
                    self.tick_state = TickState::IncrementN;
                }

                let in_range = Oam::in_range(current_scanline, y_pos);
                let addr = self.oam2_index * 4;
                self.oam2[addr] = y_pos;

                if in_range {
                    self.tick_state = TickState::ReadTileIndex;
                } else {
                    self.tick_state = TickState::IncrementN;
                }
            }
            TickState::ReadTileIndex => {
                let addr = self.addr + self.n * 4 + 1;
                let tile_idx = self.oam[addr];
                self.tick_state = TickState::WriteTileIndex(tile_idx);
            }
            TickState::WriteTileIndex(tile_idx) => {
                let addr = self.oam2_index * 4 + 1;
                self.oam2[addr] = tile_idx;
                self.tick_state = TickState::ReadAttributes;
            }
            TickState::ReadAttributes => {
                let addr = self.addr + self.n * 4 + 2;
                let attributes = self.oam[addr];
                self.tick_state = TickState::WriteAttributes(attributes);
            }
            TickState::WriteAttributes(attributes) => {
                let addr = self.oam2_index * 4 + 2;
                self.oam2[addr] = attributes;
                self.tick_state = TickState::ReadXPosition;
            }
            TickState::ReadXPosition => {
                let addr = self.addr + self.n * 4 + 3;
                let x_pos = self.oam[addr];
                self.tick_state = TickState::WriteXPosition(x_pos);
            }
            TickState::WriteXPosition(x_pos) => {
                let addr = self.oam2_index * 4 + 3;
                self.oam2[addr] = x_pos;
                self.oam2_index += 1;
                self.tick_state = TickState::IncrementN;
            }
            TickState::IncrementN => {
                self.increment_n();

                if self.n == 0 {
                    self.tick_state = TickState::BusyLoop;
                } else if self.oam2_index < 7 {
                    self.tick_state = TickState::ReadYPosition;
                } else {
                    self.tick_state = TickState::OverflowDetection(0);
                }

                // This bookkeeping does not take a cycle
                self.sprite_eval(current_scanline);
            }
            TickState::OverflowDetection(m) => {
                let addr = self.addr + self.n * 4 + m;
                let buggy_y_pos = self.oam[addr];
                if Oam::in_range(current_scanline, buggy_y_pos) {
                    // set sprite overflow
                    ret = true;
                    self.tick_state = TickState::Phase3ExtraRead(1);
                } else {
                    if m < 3 {
                        self.tick_state = TickState::OverflowDetection(m + 1);
                    } else {
                        self.increment_n();
                        if self.n == 0 {
                            self.tick_state = TickState::BusyLoop;
                        } else {
                            self.tick_state = TickState::OverflowDetection(0);
                        }
                    }
                }
            }
            TickState::Phase3ExtraRead(m) => {
                if m < 4 {
                    self.tick_state = TickState::Phase3ExtraRead(m + 1);
                } else {
                    self.increment_n();
                    self.tick_state = TickState::BusyLoop;
                }
            }
            TickState::BusyLoop => self.increment_n(),
        }

        ret
    }

    fn increment_n(&mut self) {
        self.n = (self.n + 1) % 64;
    }
}

impl Mem for Oam {
    fn peekb(&self, addr: u16) -> u8 {
        self.oam[addr as usize]
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        self.oam[addr as usize] = val
    }
}
