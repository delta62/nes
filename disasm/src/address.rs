use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
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

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Address::*;

        match self {
            Absolute(addr) => write!(f, "${:04X}", addr),
            AbsoluteX(addr) => write!(f, "${:04X},X", addr),
            AbsoluteY(addr) => write!(f, "${:04X},Y", addr),
            Immediate(addr) => write!(f, "#${:02X}", addr),
            Implied => Ok(()),
            Indirect(addr) => write!(f, "(${:04X})", addr),
            IndirectX(addr) => write!(f, "(${:02X},X)", addr),
            IndirectY(addr) => write!(f, "(${:02X}),Y", addr),
            Relative(offset) => write!(f, "*{:+}", offset),
            ZeroPage(addr) => write!(f, "${:02X}", addr),
            ZeroPageX(addr) => write!(f, "${:02X},X", addr),
            ZeroPageY(addr) => write!(f, "${:02X},Y", addr),
        }
    }
}

impl Address {
    pub fn from_bytes(op: u8, lo: u8, hi: u8) -> Self {
        let addr16 = ((hi as u16) << 8) | (lo as u16);

        // Edge cases
        match op {
            0x20 => return Address::Absolute(addr16),
            0x9E | 0xBE | 0xBF | 0x9F => return Address::AbsoluteY(addr16),
            0x00 | 0x40 | 0x60 => return Address::Implied,
            0x02 | 0x22 | 0x42 | 0x62 => return Address::Implied,
            0x6C => return Address::Indirect(addr16),
            0x96 | 0x97 | 0xB6 | 0xB7 => return Address::ZeroPageY(lo),
            _ => {}
        }

        match op % 0x20 {
            0x0C..=0x0F => Address::Absolute(addr16),
            0x1C..=0x1F => Address::AbsoluteX(addr16),
            0x19 | 0x1B => Address::AbsoluteY(addr16),
            0x00 | 0x02 | 0x09 | 0x0B => Address::Immediate(lo),
            0x08 | 0x0A | 0x12 | 0x18 | 0x1A => Address::Implied,
            0x01 | 0x03 => Address::IndirectX(lo),
            0x11 | 0x13 => Address::IndirectY(lo),
            0x10 => Address::Relative(lo as i8),
            0x04..=0x07 => Address::ZeroPage(lo),
            0x14..=0x17 => Address::ZeroPageX(lo),
            _ => unreachable!(),
        }
    }

    pub fn absolute_from_le(lo: u8, hi: u8) -> Self {
        let addr16 = ((hi as u16) << 8) | (lo as u16);
        Self::Absolute(addr16)
    }

    pub fn byte_len(&self) -> usize {
        use Address::*;
        match self {
            Absolute(_) => 2,
            AbsoluteX(_) => 2,
            AbsoluteY(_) => 2,
            Immediate(_) => 1,
            Implied => 0,
            Indirect(_) => 2,
            IndirectX(_) => 1,
            IndirectY(_) => 1,
            Relative(_) => 1,
            ZeroPage(_) => 1,
            ZeroPageX(_) => 1,
            ZeroPageY(_) => 1,
        }
    }

    pub fn byte1(&self) -> Option<u8> {
        use Address::*;
        match *self {
            Absolute(addr) | AbsoluteX(addr) | AbsoluteY(addr) | Indirect(addr) => {
                Some((addr & 0x00FF) as u8)
            }
            Immediate(val) | IndirectX(val) | IndirectY(val) | ZeroPage(val) | ZeroPageX(val)
            | ZeroPageY(val) => Some(val),
            Relative(offset) => Some(offset as u8),
            Implied => None,
        }
    }

    pub fn byte2(&self) -> Option<u8> {
        use Address::*;
        match *self {
            Absolute(addr) | AbsoluteX(addr) | AbsoluteY(addr) | Indirect(addr) => {
                Some(((addr & 0xFF00) >> 8) as u8)
            }
            Immediate(_) | IndirectX(_) | IndirectY(_) | ZeroPage(_) | ZeroPageX(_)
            | ZeroPageY(_) | Relative(_) => None,
            Implied => None,
        }
    }
}
