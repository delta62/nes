use crate::{address::Address, disassembler::PRG_ROM_BASE};
use log::warn;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mnemonic {
    ADC,
    AHX,
    ANC,
    ALR,
    AND,
    ARR,
    ASL,
    AXS,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DCP,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    ISB, // This is internally represented as ISC
    JMP,
    JSR,
    LAS,
    LAX,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    RLA,
    ROL,
    ROR,
    RRA,
    RTI,
    RTS,
    SAX,
    SBC,
    SEC,
    SED,
    SEI,
    SHX,
    SHY,
    SLO,
    SRE,
    STA,
    STP,
    STX,
    STY,
    TAS,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    XAA,
}

impl From<u8> for Mnemonic {
    fn from(value: u8) -> Self {
        use Mnemonic::*;

        match value {
            0x69 | 0x65 | 0x75 | 0x61 | 0x71 | 0x6D | 0x7D | 0x79 => ADC,
            0x93 | 0x9F => AHX,
            0x0B | 0x2B => ANC,
            0x4B => ALR,
            0x29 | 0x25 | 0x35 | 0x21 | 0x31 | 0x2D | 0x3D | 0x39 => AND,
            0x6B => ARR,
            0x0A | 0x06 | 0x16 | 0x0E | 0x1E => ASL,
            0xCB => AXS,
            0x90 => BCC,
            0xB0 => BCS,
            0xF0 => BEQ,
            0x24 | 0x2C => BIT,
            0x30 => BMI,
            0xD0 => BNE,
            0x10 => BPL,
            0x00 => BRK,
            0x50 => BVC,
            0x70 => BVS,
            0x18 => CLC,
            0xD8 => CLD,
            0x58 => CLI,
            0xB8 => CLV,
            0xC9 | 0xC5 | 0xD5 | 0xC1 | 0xD1 | 0xCD | 0xDD | 0xD9 => CMP,
            0xE0 | 0xE4 | 0xEC => CPX,
            0xC0 | 0xC4 | 0xCC => CPY,
            0xC7 | 0xD7 | 0xC3 | 0xD3 | 0xCF | 0xDF | 0xDB => DCP,
            0xC6 | 0xD6 | 0xCE | 0xDE => DEC,
            0xCA => DEX,
            0x88 => DEY,
            0x49 | 0x45 | 0x55 | 0x41 | 0x51 | 0x4D | 0x5D | 0x59 => EOR,
            0xE6 | 0xF6 | 0xEE | 0xFE => INC,
            0xE8 => INX,
            0xC8 => INY,
            0xE7 | 0xF7 | 0xE3 | 0xF3 | 0xEF | 0xFF | 0xFB => ISB, // This is internally represented as ISC
            0x4C | 0x6C => JMP,
            0x20 => JSR,
            0xBB => LAS,
            0xA7 | 0xB7 | 0xA3 | 0xB3 | 0xAB | 0xAF | 0xBF => LAX,
            0xA9 | 0xA5 | 0xB5 | 0xA1 | 0xB1 | 0xAD | 0xBD | 0xB9 => LDA,
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => LDX,
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => LDY,
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E => LSR,
            0xEA | 0x04 | 0x0C | 0x14 | 0x1A | 0x1C | 0x34 | 0x3A | 0x3C | 0x44 | 0x54 | 0x5A
            | 0x5C | 0x64 | 0x74 | 0x7A | 0x7C | 0x80 | 0x82 | 0x89 | 0xC2 | 0xD4 | 0xDA | 0xDC
            | 0xE2 | 0xF4 | 0xFA | 0xFC => NOP,
            0x09 | 0x05 | 0x15 | 0x01 | 0x11 | 0x0D | 0x1D | 0x19 => ORA,
            0x48 => PHA,
            0x08 => PHP,
            0x68 => PLA,
            0x28 => PLP,
            0x27 | 0x37 | 0x23 | 0x33 | 0x2F | 0x3F | 0x3B => RLA,
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E => ROL,
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E => ROR,
            0x67 | 0x77 | 0x63 | 0x73 | 0x6F | 0x7F | 0x7B => RRA,
            0x40 => RTI,
            0x60 => RTS,
            0x87 | 0x97 | 0x83 | 0x8F => SAX,
            0xE9 | 0xE5 | 0xF5 | 0xE1 | 0xF1 | 0xED | 0xFD | 0xF9 | 0xEB => SBC,
            0x38 => SEC,
            0xF8 => SED,
            0x78 => SEI,
            0x9E => SHX,
            0x9C => SHY,
            0x07 | 0x17 | 0x03 | 0x13 | 0x0F | 0x1F | 0x1B => SLO,
            0x47 | 0x57 | 0x43 | 0x53 | 0x4F | 0x5F | 0x5B => SRE,
            0x85 | 0x95 | 0x81 | 0x91 | 0x8D | 0x9D | 0x99 => STA,
            0x02 | 0x22 | 0x42 | 0x62 | 0x12 | 0x32 | 0x52 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2 => {
                STP
            }
            0x86 | 0x96 | 0x8E => STX,
            0x84 | 0x94 | 0x8C => STY,
            0x9B => TAS,
            0xAA => TAX,
            0xA8 => TAY,
            0xBA => TSX,
            0x8A => TXA,
            0x9A => TXS,
            0x98 => TYA,
            0x8B => XAA,
        }
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Mnemonic::*;

        match self {
            ADC => write!(f, "ADC"),
            AHX => write!(f, "AHX"),
            ANC => write!(f, "ANC"),
            ALR => write!(f, "ALR"),
            AND => write!(f, "AND"),
            ARR => write!(f, "ARR"),
            ASL => write!(f, "ASL"),
            AXS => write!(f, "AXS"),
            BCC => write!(f, "BCC"),
            BCS => write!(f, "BCS"),
            BEQ => write!(f, "BEQ"),
            BIT => write!(f, "BIT"),
            BMI => write!(f, "BMI"),
            BNE => write!(f, "BNE"),
            BPL => write!(f, "BPL"),
            BRK => write!(f, "BRK"),
            BVC => write!(f, "BVC"),
            BVS => write!(f, "BVS"),
            CLC => write!(f, "CLC"),
            CLD => write!(f, "CLD"),
            CLI => write!(f, "CLI"),
            CLV => write!(f, "CLV"),
            CMP => write!(f, "CMP"),
            CPX => write!(f, "CPX"),
            CPY => write!(f, "CPY"),
            DCP => write!(f, "DCP"),
            DEC => write!(f, "DEC"),
            DEX => write!(f, "DEX"),
            DEY => write!(f, "DEY"),
            EOR => write!(f, "EOR"),
            INC => write!(f, "INC"),
            INX => write!(f, "INX"),
            INY => write!(f, "INY"),
            ISB => write!(f, "ISB"), // This is internally represented as ISC
            JMP => write!(f, "JMP"),
            JSR => write!(f, "JSR"),
            LAS => write!(f, "LAS"),
            LAX => write!(f, "LAX"),
            LDA => write!(f, "LDA"),
            LDX => write!(f, "LDX"),
            LDY => write!(f, "LDY"),
            LSR => write!(f, "LSR"),
            NOP => write!(f, "NOP"),
            ORA => write!(f, "ORA"),
            PHA => write!(f, "PHA"),
            PHP => write!(f, "PHP"),
            PLA => write!(f, "PLA"),
            PLP => write!(f, "PLP"),
            RLA => write!(f, "RLA"),
            ROL => write!(f, "ROL"),
            ROR => write!(f, "ROR"),
            RRA => write!(f, "RRA"),
            RTI => write!(f, "RTI"),
            RTS => write!(f, "RTS"),
            SAX => write!(f, "SAX"),
            SBC => write!(f, "SBC"),
            SEC => write!(f, "SEC"),
            SED => write!(f, "SED"),
            SEI => write!(f, "SEI"),
            SHX => write!(f, "SHX"),
            SHY => write!(f, "SHY"),
            SLO => write!(f, "SLO"),
            SRE => write!(f, "SRE"),
            STA => write!(f, "STA"),
            STP => write!(f, "STP"),
            STX => write!(f, "STX"),
            STY => write!(f, "STY"),
            TAS => write!(f, "TAS"),
            TAX => write!(f, "TAX"),
            TAY => write!(f, "TAY"),
            TSX => write!(f, "TSX"),
            TXA => write!(f, "TXA"),
            TXS => write!(f, "TXS"),
            TYA => write!(f, "TYA"),
            XAA => write!(f, "XAA"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Op {
    pub opcode: u8,
    pub mnemonic: Mnemonic,
    pub addr: Address,
}

impl Op {
    pub fn from_bytes(opcode: u8, next1: u8, next2: u8) -> Self {
        let addr = Address::from_bytes(opcode, next1, next2);
        let mnemonic = opcode.into();
        Self {
            opcode,
            addr,
            mnemonic,
        }
    }

    pub fn is_undocumented(&self) -> bool {
        use Mnemonic::*;

        // There is one version of SBC which is undocumented
        if self.opcode == 0xEB {
            return true;
        }

        // There are many versions of NOP, but only $EA is documented
        if self.mnemonic == NOP {
            return self.opcode != 0xEA;
        }

        matches!(
            self.mnemonic,
            SLO | ANC
                | RLA
                | SRE
                | ALR
                | RRA
                | ARR
                | SAX
                | XAA
                | AHX
                | TAS
                | LAX
                | LAS
                | DCP
                | AXS
                | ISB
        )
    }

    pub fn byte_len(&self) -> usize {
        // opcode byte + addr
        self.addr.byte_len() + 1
    }

    /// Each opcode may result in control flow going to 0-2 other instructions. For example:
    /// - STP will never go anywhere else
    /// - JMP will always go to one location
    /// - BNE might go to one of two different locations
    ///
    /// This function resolves the next locations in memory (if any) that control flow will
    /// go and returns them as a tuple. If there are two next locations, the order of locations
    /// in the tuple is undefined.
    pub fn next_addresses(&self, from: usize) -> (Option<usize>, Option<usize>) {
        use Mnemonic::*;

        match self.mnemonic {
            JMP => match self.addr {
                Address::Absolute(addr) => {
                    let jump_target = addr as usize;

                    if jump_target >= PRG_ROM_BASE {
                        (Some(jump_target), None)
                    } else {
                        warn!("Skipping disassembly of JMP call to {addr:04X}");
                        (None, None)
                    }
                }
                Address::Indirect(addr) => {
                    // This will change live at runtime; trying to interpret uninitialized memory
                    // might just lead to garbage output.
                    warn!("Skipping disassembly of indirect call to {addr:04X}");
                    (None, None)
                }
                _ => unreachable!("JMP only uses absolute or indirect addressing"),
            },
            JSR => {
                if let Address::Absolute(addr) = self.addr {
                    let next = from + self.byte_len() + 1;
                    let jump_target = addr as usize;
                    if jump_target >= PRG_ROM_BASE {
                        (Some(next), Some(jump_target))
                    } else {
                        warn!("Skipping disassembly of JSR call to {addr:04X}");
                        (Some(next), None)
                    }
                } else {
                    unreachable!("JSR only uses absolute addressing");
                }
            }
            BCC | BCS | BEQ | BMI | BNE | BPL | BVC | BVS => {
                if let Address::Relative(offset) = self.addr {
                    let next = from + self.byte_len();
                    let jump_target = if offset.is_negative() {
                        let offset = -offset as u16;
                        let from = from as u16 + 2;
                        from.wrapping_sub(offset)
                    } else {
                        let offset = offset as u16;
                        let from = from as u16 + 2;
                        from.wrapping_add(offset)
                    };

                    if jump_target as usize >= PRG_ROM_BASE {
                        (Some(next), Some(jump_target as usize))
                    } else {
                        warn!("Skipping disassembly of branch call to {jump_target:04X}");
                        (Some(next), None)
                    }
                } else {
                    unreachable!("Branch conditions only use relative addressing");
                }
            }
            STP => (None, None),
            _ => (Some(from + self.byte_len()), None),
        }
    }
}
