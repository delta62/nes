use crate::disasm::{Address, Op};
use nes::{Cpu, Mem, Nes};

/// Convert two arbitrary sized numbers into a 16 bit word
/// It only really makes sense when using u8 or potentially u16
macro_rules! word {
    (
        $lo:expr,
        $hi:expr
    ) => {
        (($hi as u16) << 8) | ($lo as u16)
    }
}

pub struct NesLogger;

impl NesLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn log(&self, nes: &Nes) -> String {
        let cpu = nes.cpu();
        let ppu = nes.ppu().borrow();

        let opcode = cpu.peekb(cpu.pc());
        let next1 = cpu.peekb(cpu.pc().wrapping_add(1));
        let next2 = cpu.peekb(cpu.pc().wrapping_add(2));
        let op = Op::from_bytes(opcode, next1, next2);

        format!(
            "{:04X}  {} {}{} {:27} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
            cpu.pc(),
            fmt_op_mem(&op),
            if op.is_undocumented() { '*' } else { ' ' },
            op.name(),
            fmt_op_addr(&op, &cpu),
            cpu.a(),
            cpu.x(),
            cpu.y(),
            cpu.flags(),
            cpu.s(),
            ppu.scanline(),
            ppu.pixel(),
            cpu.cy(),
        )
    }
}

fn fmt_op_mem(op: &Op) -> String {
    match op.addr {
        Address::Absolute(_) | Address::AbsoluteX(_) | Address::AbsoluteY(_) | Address::Indirect(_) => {
            format!("{:02X} {:02X} {:02X}", op.code, op.next1, op.next2)
        }
        Address::Implied => format!("{:02X}      ", op.code),
        // other addressing modes use 1 byte arg
        _ => format!("{:02X} {:02X}   ", op.code, op.next1),
    }
}

fn fmt_op_addr<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    let addr = match op.addr {
        Address::Absolute(a) => format!("${:04X}", a),
        Address::AbsoluteX(a) => format!("${:04X},X", a),
        Address::AbsoluteY(a) => format!("${:04X},Y", a),
        Address::Immediate(i) => format!("#${:02X}", i),
        Address::ZeroPage(z) => format!("${:02X}", z),
        Address::ZeroPageX(z) => format!("${:02X},X", z),
        Address::ZeroPageY(z) => format!("${:02X},Y", z),
        Address::Implied => match op.code {
            // LSR, ASL, ROR, ROL use acc in implied mode
            0x4A | 0x0A | 0x6A | 0x2A => "A".to_owned(),
            _ => "".to_owned(),
        },
        Address::Relative(r) => format!("${:04X}", (cpu.pc() as i32) + r as i32 + 2),
        Address::Indirect(i) => format!("(${:04X})", i),
        Address::IndirectX(z) => {
            let zp_addr = cpu.x().wrapping_add(z);
            let lo = cpu.peekb(zp_addr as u16);
            let hi = cpu.peekb(zp_addr.wrapping_add(1) as u16);
            let data_addr = word!(lo, hi);
            let data = cpu.peekb(data_addr);
            format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", z, zp_addr, data_addr, data)
        }
        Address::IndirectY(z) => {
            let lo = cpu.peekb(z as u16);
            let hi = cpu.peekb(z.wrapping_add(1) as u16);
            let data_addr = word!(lo, hi);
            let inc_addr = data_addr.wrapping_add(cpu.y() as u16);
            let data = cpu.peekb(inc_addr);
            format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", z, data_addr, inc_addr, data)
        }
    };
    let extra = fmt_op_extra(&op, cpu);

    format!("{}{}", addr, extra)
}

fn fmt_zp<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    format!(" = {:02X}", cpu.peekb(op.next1 as u16))
}

fn fmt_abs<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    format!(" = {:02X}", cpu.peekb(word!(op.next1, op.next2)))
}

fn fmt_indirect_word<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    if op.next1 == 0xFF {
        let page = (op.next2 as u16) << 8;
        let lo = cpu.peekb(page + 0x00FF);
        let hi = cpu.peekb(page);
        format!(" = {:02X}{:02X}", hi, lo)
    } else {
        format!(" = {:04X}", cpu.peekw(word!(op.next1, op.next2)))
    }
}

fn fmt_zp_x<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    let addr = op.next1.wrapping_add(cpu.x());
    let data = cpu.peekb(addr as u16);
    format!(" @ {:02X} = {:02X}", addr, data)
}

fn fmt_zp_y<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    let addr = op.next1.wrapping_add(cpu.y());
    let data = cpu.peekb(addr as u16);
    format!(" @ {:02X} = {:02X}", addr, data)
}

fn fmt_abs_x<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    let addr = word!(op.next1, op.next2).wrapping_add(cpu.x() as u16);
    let data = cpu.peekb(addr);
    format!(" @ {:04X} = {:02X}", addr, data)
}

fn fmt_abs_y<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    let addr = word!(op.next1, op.next2).wrapping_add(cpu.y() as u16);
    let data = cpu.peekb(addr);
    format!(" @ {:04X} = {:02X}", addr, data)
}

fn fmt_op_extra<M: Mem>(op: &Op, cpu: &Cpu<M>) -> String {
    match op.code {
        0x6C => fmt_indirect_word(op, cpu),
        0xAC | 0x8C | 0x8D | 0x8E | 0xAE | 0xAD | 0x2C | 0x0D | 0x2D |
        0x4D | 0x6D | 0xCD | 0xED | 0xEC | 0xCC | 0x4E | 0x0E | 0x6E |
        0x2E | 0xEE | 0xCE | 0x0C | 0xAF | 0x8F | 0xCF | 0xEF | 0x0F |
        0x2F | 0x4F | 0x6F => fmt_abs(op, cpu),
        0xBC | 0x1D | 0x3D | 0x5D | 0x7D | 0xDD | 0xFD | 0xBD | 0x9D |
        0x5E | 0x1E | 0x7E | 0x3E | 0xFE | 0xDE | 0x1c | 0x3C | 0x5C |
        0x7C | 0xDC | 0xFC | 0xDF | 0xFF | 0x1F | 0x3F | 0x5F | 0x7F => fmt_abs_x(op, cpu),
        0xB9 | 0x19 | 0x39 | 0x59 | 0x79 | 0xD9 | 0xF9 | 0x99 | 0xBE |
        0xBF | 0xDB | 0xFB | 0x1B | 0x3B | 0x5B | 0x7B => fmt_abs_y(op, cpu),
        0x24 | 0x85 | 0x86 | 0xA5 | 0xA4 | 0x84 | 0xA6 | 0x05 | 0x25 |
        0x45 | 0x65 | 0xC5 | 0xE5 | 0xE4 | 0xC4 | 0x46 | 0x06 | 0x66 |
        0x26 | 0xE6 | 0xC6 | 0x04 | 0x44 | 0x64 | 0xA7 | 0x87 | 0xC7 |
        0xE7 | 0x07 | 0x27 | 0x47 | 0x67 => fmt_zp(op, cpu),
        0xB4 | 0x94 | 0x15 | 0x35 | 0x55 | 0x75 | 0xD5 | 0xF5 | 0xB5 |
        0x95 | 0x56 | 0x16 | 0x76 | 0x36 | 0xF6 | 0xD6 | 0x14 | 0x34 |
        0x54 | 0x74 | 0xD4 | 0xF4 | 0xD7 | 0xF7 | 0x17 | 0x37 | 0x57 |
        0x77 => fmt_zp_x(op, cpu),
        0xB6 | 0x96 | 0xB7 | 0x97 => fmt_zp_y(op, cpu),
        _ => "".to_owned(),
    }
}


