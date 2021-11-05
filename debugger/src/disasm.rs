pub enum Address {
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    Immediate(u8),
    Implied,
    Indirect(u16),
    IndirectX(u8),
    IndirectY(u8),
    Relative(i8),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
}

impl Address {
    fn from_bytes(op: u8, lo: u8, hi: u8) -> Self {
        let addr16 = ((hi as u16) << 8) | (lo as u16);

        // Edge cases
        match op {
            0x20                      => return Address::Absolute(addr16),
            0x9E | 0xBE | 0xBF | 0x9F => return Address::AbsoluteY(addr16),
            0x00 | 0x40 | 0x60        => return Address::Implied,
            0x02 | 0x22 | 0x42 | 0x62 => return Address::Implied,
            0x6C                      => return Address::Indirect(addr16),
            0x96 | 0x97 | 0xB6 | 0xB7 => return Address::ZeroPageY(lo),
            _ => { }
        }

        match op % 0x20 {
            0x0C | 0x0D | 0x0E | 0x0F        => Address::Absolute(addr16),
            0x1C | 0x1D | 0x1E | 0x1F        => Address::AbsoluteX(addr16),
            0x19 | 0x1B                      => Address::AbsoluteY(addr16),
            0x00 | 0x02 | 0x09 | 0x0B        => Address::Immediate(lo),
            0x08 | 0x0A | 0x12 | 0x18 | 0x1A => Address::Implied,
            0x01 | 0x03                      => Address::IndirectX(lo),
            0x11 | 0x13                      => Address::IndirectY(lo),
            0x10                             => Address::Relative(lo as i8),
            0x04 | 0x05 | 0x06 | 0x07        => Address::ZeroPage(lo),
            0x14 | 0x15 | 0x16 | 0x17        => Address::ZeroPageX(lo),
            _                                => unreachable!(),
        }
    }
}

pub struct Op {
    pub addr: Address,
    pub code: u8,
    pub next1: u8,
    pub next2: u8,
}

impl Op {
    pub fn from_bytes(code: u8, next1: u8, next2: u8) -> Self {
        let addr = Address::from_bytes(code, next1, next2);

        Self { code, addr, next1, next2 }
    }

    pub fn name(&self) -> &'static str {
        match self.code {
            0x69 | 0x65 | 0x75 | 0x61 | 0x71 | 0x6D | 0x7D | 0x79 => "ADC",
            0x29 | 0x25 | 0x35 | 0x21 | 0x31 | 0x2D | 0x3D | 0x39 => "AND",
            0x0A | 0x06 | 0x16 | 0x0E | 0x1E                      => "ASL",
            0x90                                                  => "BCC",
            0xB0                                                  => "BCS",
            0xF0                                                  => "BEQ",
            0x24 | 0x2C                                           => "BIT",
            0x30                                                  => "BMI",
            0xD0                                                  => "BNE",
            0x10                                                  => "BPL",
            0x00                                                  => "BRK",
            0x50                                                  => "BVC",
            0x70                                                  => "BVS",
            0x18                                                  => "CLC",
            0xD8                                                  => "CLD",
            0xB8                                                  => "CLV",
            0xC9 | 0xC5 | 0xD5 | 0xC1 | 0xD1 | 0xCD | 0xDD | 0xD9 => "CMP",
            0xE0 | 0xE4 | 0xEC                                    => "CPX",
            0xC0 | 0xC4 | 0xCC                                    => "CPY",
            0xC7 | 0xD7 | 0xC3 | 0xD3 | 0xCF | 0xDF | 0xDB        => "DCP",
            0xC6 | 0xD6 | 0xCE | 0xDE                             => "DEC",
            0xCA                                                  => "DEX",
            0x88                                                  => "DEY",
            0x49 | 0x45 | 0x55 | 0x41 | 0x51 | 0x4D | 0x5D | 0x59 => "EOR",
            0xE6 | 0xF6 | 0xEE | 0xFE                             => "INC",
            0xE8                                                  => "INX",
            0xC8                                                  => "INY",
            0xE7 | 0xF7 | 0xE3 | 0xF3 | 0xEF | 0xFF | 0xFB        => "ISB", // This is internally represented as ISC
            0x4C | 0x6C                                           => "JMP",
            0x20                                                  => "JSR",
            0xA7 | 0xB7 | 0xA3 | 0xB3 | 0xAF | 0xBF               => "LAX",
            0xA9 | 0xA5 | 0xB5 | 0xA1 | 0xB1 | 0xAD | 0xBD | 0xB9 => "LDA",
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE                      => "LDX",
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC                      => "LDY",
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E                      => "LSR",
            0xEA | 0x04 | 0x0C | 0x14 | 0x1A | 0x1C | 0x34 |
            0x3A | 0x3C | 0x44 | 0x54 | 0x5A | 0x5C | 0x64 |
            0x74 | 0x7A | 0x7C | 0x80 | 0x82 | 0x89 | 0xC2 |
            0xD4 | 0xDA | 0xDC | 0xE2 | 0xF4 | 0xFA | 0xFC        => "NOP",
            0x09 | 0x05 | 0x15 | 0x01 | 0x11 | 0x0D | 0x1D | 0x19 => "ORA",
            0x48                                                  => "PHA",
            0x08                                                  => "PHP",
            0x68                                                  => "PLA",
            0x28                                                  => "PLP",
            0x27 | 0x37 | 0x23 | 0x33 | 0x2F | 0x3F | 0x3B        => "RLA",
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E                      => "ROL",
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E                      => "ROR",
            0x67 | 0x77 | 0x63 | 0x73 | 0x6F | 0x7F | 0x7B        => "RRA",
            0x40                                                  => "RTI",
            0x60                                                  => "RTS",
            0x87 | 0x97 | 0x83 | 0x8F                             => "SAX",
            0xE9 | 0xE5 | 0xF5 | 0xE1 | 0xF1 | 0xED | 0xFD | 0xF9 |
            0xEB                                                  => "SBC",
            0x38                                                  => "SEC",
            0xF8                                                  => "SED",
            0x78                                                  => "SEI",
            0x07 | 0x17 | 0x03 | 0x13 | 0x0F | 0x1F | 0x1B        => "SLO",
            0x47 | 0x57 | 0x43 | 0x53 | 0x4F | 0x5F | 0x5B        => "SRE",
            0x85 | 0x95 | 0x81 | 0x91 | 0x8D | 0x9D | 0x99        => "STA",
            0x86 | 0x96 | 0x8E                                    => "STX",
            0x84 | 0x94 | 0x8C                                    => "STY",
            0xAA                                                  => "TAX",
            0xA8                                                  => "TAY",
            0xBA                                                  => "TSX",
            0x8A                                                  => "TXA",
            0x9A                                                  => "TXS",
            0x98                                                  => "TYA",
            unknown                                               => panic!("Unknown opcode {:02X}", unknown),
        }
    }

    pub fn is_undocumented(&self) -> bool {
        match self.code % 0x20 {
            0x12 | 0x03 | 0x07 | 0x0B | 0x0F | 0x13 | 0x17 | 0x1B |
            0x1F => return true,
            _ => { }
        }

        match self.code {
            0x80 | 0x04 | 0x44 | 0x64 | 0x0C | 0x14 | 0x34 | 0x54 |
            0x74 | 0xD4 | 0xF4 | 0x1C | 0x3C | 0x5C | 0x7C | 0x9C |
            0xDC | 0xFC | 0x89 | 0x02 | 0x22 | 0x42 | 0x62 | 0x82 |
            0xC2 | 0xE2 | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA |
            0x9E => true,
            _ => false,
        }
    }
}
