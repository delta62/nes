use bitflags::bitflags;
use crate::mem::Mem;
use getset::{CopyGetters, Getters};

/// The size (in bytes) of one page in memory.
const PAGE_SIZE: u16 = 0x0100;

/// The memory page that the stack uses. Under normal operating conditions the
/// stack pointer is always between 0x0100 and 0x01FF.
const STACK_BASE: u16 = 0x0100;


/// When an NMI occurs, the program counter is updated to the value stored in
/// this memory address.
const NMI_VECTOR: u16 = 0xFFFA;

/// When the CPU is reset, the program counter is updated to the value stored in
/// this memory address.
const RESET_VECTOR: u16 = 0xFFFC;

/// When an interrupt occurs, the program counter is updated to the value stored
/// in this address
const BRK_VECTOR: u16 = 0xFFFE;

/// Configures opcodes and points them to functions on self. For each operation
/// that should be bound, provide the opcode, the function to invoke on self,
/// the addressing mode to invoke on self, and the number of cycles that this
/// instruction takes (not including page transitions or branch instructions).
///
/// The given operations are compiled into one match expression which will
/// update the CPU's internal state.
macro_rules! ops {
    (
        $this:ident,
        $test:expr,
        [
            $( ( $op:literal, $fn:ident, $addr:ident, $cycles:literal, $extra:literal ) ),*
            $(,)?
        ]
    ) => {
        match $test {
            $(
                $op => {
                    let addr = $this.$addr($extra);
                    $this.$fn(addr);
                    $this.busy = $cycles - 1;
                    $this.cy += 1;
                },
            )*
        }
    };
}

/// Represents a destination in memory to read from or write to during a CPU
/// operation. Some addressing modes may incur additional cycles when they cross
/// page boundaries or execute branch jumps. These cycles are accounted for when
/// loading variables or evaluating branch conditions.
#[derive(Clone, Copy, Debug)]
enum Address {
    Absolute(u16),
    Accumulator,
    Immediate(u8),
    Implied,
    Indirect(u16),
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub struct Flags: u8 {
        /// The carry flag is somewhat overloaded in meaning, but it typically means
        /// that during the previous operation the high bit of the operand was set and
        /// shifted out, either due to a shift operation or some mathematical process.
        /// See the documentation for the specific op code to understand what this flag
        /// means in practice.
        const CARRY = 0b0000_0001;

        /// The zero flag indicates that the last operation resulted in a zero value.
        const ZERO = 0b0000_0010;

        /// The IRQ disable flag prevents interrupts from being serviced by the CPU.
        /// Note that NMI interrupts cannot be disabled and will always run when the CPU
        /// receives them.
        const IRQ_DISABLE = 0b0000_0100;

        /// The decimal flag is not implemented on the NES, but it is still possible to
        /// set the flag.
        const DECIMAL = 0b0000_1000;

        /// The break bit isn't really a flag in that the hardware doesn't have a flag
        /// to represent a break. However, when the processor pushes its flags to the
        /// stack it pushes a single byte, and bit 4 of that byte can be enabled to
        /// indicate that the interrupt occurred as a result of the BRK opcode.
        const BREAK = 0b0001_0000;

        // const UNUSED      = 0b0010_0000;

        /// The overflow flag is typically used to represent an overflow when performing
        /// addition and subtraction, but the actual semantics depend on the opcode. See
        /// the documentation for individual opcodes to understand what this flag means
        /// in practice.
        const OVERFLOW = 0b0100_0000;

        /// The negative flag indicates that the last operation resulted in a negative
        /// two's complement value, i.e. bit 7 of the result was set.
        const NEGATIVE = 0b1000_0000;
    }
}

#[derive(CopyGetters, Getters)]
pub struct Cpu<M: Mem> {
    /// The number of CPU cycles that have elapsed since power on
    #[getset(get_copy = "pub")]
    cy: u64,

    busy: u8,

    /// The accumulator register
    #[getset(get_copy = "pub")]
    a: u8,

    /// The X register
    #[getset(get_copy = "pub")]
    x: u8,

    /// The y register
    #[getset(get_copy = "pub")]
    y: u8,

    /// The stack pointer. This contains the low byte of the stack address. The
    /// hight byte is always 0x01, and the stack grows "up", meaning that an
    /// empty stack pointer points to 0x01FF and a full stack points to 0x0100.
    #[getset(get_copy = "pub")]
    s: u8,

    /// This byte holds all of the CPU's status flags. It's useful to represent
    /// them as a byte rather than a bunch of booleans since some operations
    /// read/write them to/from the stack as a single byte.
    #[getset(get_copy = "pub")]
    flags: Flags,

    /// The program counter points to the next piece of data for the CPU to
    /// evaluate. The 6502 initializes the PC by setting it to the word stored
    /// in memory at 0xFFFC and 0xFFFD (the reset vector).
    #[getset(get_copy = "pub")]
    pc: u16,

    /// All addressable space that the CPU can access. This includes RAM, the
    /// game pak, the PPU, and so on.
    mem: M,
}

impl<M: Mem> Cpu<M> {
    /// Creates a new CPU with the given addressable components. At boot time
    /// the IRQ disable flag will be set, and the state of other registers and
    /// flags is left as undefined.
    pub fn new(mem: M) -> Cpu<M> {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            flags: Flags::from_bits_retain(0x24),
            s: 0xFD,
            pc: 0x0000,
            cy: 0,
            busy: 0,
            mem,
        }
    }

    /// Reset the CPU. This will load the program counter from the reset vector
    /// and set the IRQ disable flag.
    pub fn reset(&mut self) {
        self.pc = self.loadw(RESET_VECTOR);
        self.flags.insert(Flags::IRQ_DISABLE);
    }

    /// If true, the CPU is currently in the middle of executing an instruction
    /// and the next step will be spent without reading a new instruction. If
    /// false, the next cycle will result in a new instruction beginning.
    pub fn is_busy(&self) -> bool {
        self.busy > 0
    }

    /// Triggers a non-maskable interrupt. This causes the current program
    /// counter and status flags to be pushed to the stack, and the PC is set to
    /// the address contained in the NMI vector (0xFFFA and 0xFFFB).
    ///
    /// An NMI will always set the IRQ disable flag, and will not push the break
    /// bit to the stack when storing the flags.
    pub fn nmi(&mut self) {
        self.flags.remove(Flags::BREAK);
        self.pushw(self.pc);
        self.pushb(self.flags.bits());
        self.flags.insert(Flags::IRQ_DISABLE);
        self.pc = self.loadw(NMI_VECTOR);
    }

    /// Triggers an intterupt request. If the IRQ disable flag is set then the
    /// interrupt will be ignored. Otherwise, the current program counter and
    /// status flags are pushed to the stack, and the program counter will be
    /// initialized to the value in the NMI vector (0xFFFE and 0xFFFF).
    ///
    /// An IRQ will always set the IRQ disable flag, and will not push the break
    /// bit to the stack when storing the flags.
    pub fn irq(&mut self) {
        if self.flags.contains(Flags::IRQ_DISABLE) {
            return;
        }

        self.flags.remove(Flags::BREAK);

        self.pushw(self.pc);
        self.pushb(self.flags.bits());
        self.flags.insert(Flags::IRQ_DISABLE);
        self.pc = self.loadw(BRK_VECTOR);
    }

    /// This causes the CPU to perform a direct memory access (DMA) to the PPU,
    /// transferring one page (256 bytes) of memory from CPU address space into
    /// the PPU's internal OAM (object attribute memory) memory. Although it is
    /// possible to write one byte at a time to the PPU using the OAMADDR and
    /// OAMDATA ports (0x2003 and 0x2004 in the CPU's memory space), this would
    /// be very slow compared to transferring the memory in bulk with a DMA
    /// call.
    ///
    /// It takes about 512 cycles to copy a page of data for DMA.
    ///
    /// The memory transfer moves one page as specified by the page parameter.
    /// Assuming that page is 0xNN, the memory from 0xNN00 to 0xNNFF will be
    /// transferred to the PPU OAM.
    fn dma(&mut self, page: u8) {
        let start = (page as u16) << 8;
        let end = start + PAGE_SIZE;

        // DMA takes 513 or 514 cycles, depending if it started on an odd cycle
        if self.cy & 1 == 0 {
            self.cy += 513;
        } else {
            self.cy += 514;
        }

        for addr in start..end {
            let val = self.loadb(addr);
            self.storeb(0x2004, val);
        }
    }

    /// Runs a single CPU instruction. The instruction will be read from memory
    /// located at the current program counter value, and the stack pointer will
    /// automatically advance past the instruction as it is read.
    ///
    /// One instruction will take several cycles to complete. This emulation
    /// performs the entire instruction immediately, then adds to the cy field
    /// the number of cycles that the CPU had to run for the instruciton.
    pub fn step(&mut self) {
        if self.busy > 0 {
            self.busy -= 1;
            self.cy += 1;
            return;
        }

        let op = self.loadb_bump_pc();

        ops!(self, op, [
            (0x00, brk, imp, 7, false), (0x01, ora, izx, 6, false), (0x02, kil, imp, 2, false),
            (0x03, slo, izx, 8, false), (0x04, nop, zp0, 3, false), (0x05, ora, zp0, 3, false),
            (0x06, asl, zp0, 5, false), (0x07, slo, zp0, 5, false), (0x08, php, imp, 3, false),
            (0x09, ora, imm, 2, false), (0x0A, asl, acc, 2, false), (0x0B, anc, imm, 2, false),
            (0x0C, nop, abs, 4, false), (0x0D, ora, abs, 4, false), (0x0E, asl, abs, 6, false),
            (0x0F, slo, abs, 6, false), (0x10, bpl, rel, 2, true ), (0x11, ora, izy, 5, true ),
            (0x12, kil, imp, 2, false), (0x13, slo, izy, 8, false), (0x14, nop, zpx, 4, false),
            (0x15, ora, zpx, 4, false), (0x16, asl, zpx, 6, false), (0x17, slo, zpx, 6, false),
            (0x18, clc, imp, 2, false), (0x19, ora, aby, 4, true ), (0x1A, nop, imp, 2, false),
            (0x1B, slo, aby, 7, false), (0x1C, nop, abx, 4, true ), (0x1D, ora, abx, 4, true ),
            (0x1E, asl, abx, 7, false), (0x1F, slo, abx, 7, false), (0x20, jsr, abs, 6, false),
            (0x21, and, izx, 6, false), (0x22, kil, imp, 2, false), (0x23, rla, izx, 8, false),
            (0x24, bit, zp0, 3, false), (0x25, and, zp0, 3, false), (0x26, rol, zp0, 5, false),
            (0x27, rla, zp0, 5, false), (0x28, plp, imp, 4, false), (0x29, and, imm, 2, false),
            (0x2A, rol, acc, 2, false), (0x2B, anc, imm, 2, false), (0x2C, bit, abs, 4, false),
            (0x2D, and, abs, 4, false), (0x2E, rol, abs, 6, false), (0x2F, rla, abs, 6, false),
            (0x30, bmi, rel, 2, true ), (0x31, and, izy, 5, true ), (0x32, kil, imp, 2, false),
            (0x33, rla, izy, 8, false), (0x34, nop, zpx, 4, false), (0x35, and, zpx, 4, false),
            (0x36, rol, zpx, 6, false), (0x37, rla, zpx, 6, false), (0x38, sec, imp, 2, false),
            (0x39, and, aby, 4, true ), (0x3A, nop, imp, 2, false), (0x3B, rla, aby, 7, false),
            (0x3C, nop, abx, 4, true ), (0x3D, and, abx, 4, true ), (0x3E, rol, abx, 7, false),
            (0x3F, rla, abx, 7, false), (0x40, rti, imp, 6, false), (0x41, eor, izx, 6, false),
            (0x42, kil, imp, 2, false), (0x43, sre, izx, 8, false), (0x44, nop, zp0, 3, false),
            (0x45, eor, zp0, 3, false), (0x46, lsr, zp0, 5, false), (0x47, sre, zp0, 5, false),
            (0x48, pha, imp, 3, false), (0x49, eor, imm, 2, false), (0x4A, lsr, acc, 2, false),
            (0x4B, alr, imm, 2, false), (0x4C, jmp, abs, 3, false), (0x4D, eor, abs, 4, false),
            (0x4E, lsr, abs, 6, false), (0x4F, sre, abs, 6, false), (0x50, bvc, rel, 2, true ),
            (0x51, eor, izy, 5, true ), (0x52, kil, imp, 2, false), (0x53, sre, izy, 8, false),
            (0x54, nop, zpx, 4, false), (0x55, eor, zpx, 4, false), (0x56, lsr, zpx, 6, false),
            (0x57, sre, zpx, 6, false), (0x58, cli, imp, 2, false), (0x59, eor, aby, 4, true ),
            (0x5A, nop, imp, 2, false), (0x5B, sre, aby, 7, false), (0x5C, nop, abx, 4, true ),
            (0x5D, eor, abx, 4, true ), (0x5E, lsr, abx, 7, false), (0x5F, sre, abx, 7, false),
            (0x60, rts, imp, 6, false), (0x61, adc, izx, 6, false), (0x62, kil, imp, 2, false),
            (0x63, rra, izx, 8, false), (0x64, nop, zp0, 3, false), (0x65, adc, zp0, 3, false),
            (0x66, ror, zp0, 5, false), (0x67, rra, zp0, 5, false), (0x68, pla, imp, 4, false),
            (0x69, adc, imm, 2, false), (0x6A, ror, acc, 2, false), (0x6B, arr, imm, 2, false),
            (0x6C, jmp, ind, 5, false), (0x6D, adc, abs, 4, false), (0x6E, ror, abs, 6, false),
            (0x6F, rra, abs, 6, false), (0x70, bvs, rel, 2, true ), (0x71, adc, izy, 5, true ),
            (0x72, kil, imp, 2, false), (0x73, rra, izy, 8, false), (0x74, nop, zpx, 4, false),
            (0x75, adc, zpx, 4, false), (0x76, ror, zpx, 6, false), (0x77, rra, zpx, 6, false),
            (0x78, sei, imp, 2, false), (0x79, adc, aby, 4, true ), (0x7A, nop, imp, 2, false),
            (0x7B, rra, aby, 7, false), (0x7C, nop, abx, 4, true ), (0x7D, adc, abx, 4, true ),
            (0x7E, ror, abx, 7, false), (0x7F, rra, abx, 7, false), (0x80, nop, imm, 2, false),
            (0x81, sta, izx, 6, false), (0x82, nop, imm, 2, false), (0x83, sax, izx, 6, false),
            (0x84, sty, zp0, 3, false), (0x85, sta, zp0, 3, false), (0x86, stx, zp0, 3, false),
            (0x87, sax, zp0, 3, false), (0x88, dey, imp, 2, false), (0x89, nop, imm, 2, false),
            (0x8A, txa, imp, 2, false), (0x8B, xaa, imm, 2, false), (0x8C, sty, abs, 4, false),
            (0x8D, sta, abs, 4, false), (0x8E, stx, abs, 4, false), (0x8F, sax, abs, 4, false),
            (0x90, bcc, rel, 2, true ), (0x91, sta, izy, 6, false), (0x92, kil, imp, 2, false),
            (0x93, ahx, izy, 6, false), (0x94, sty, zpx, 4, false), (0x95, sta, zpx, 4, false),
            (0x96, stx, zpy, 4, false), (0x97, sax, zpy, 4, false), (0x98, tya, imp, 2, false),
            (0x99, sta, aby, 5, false), (0x9A, txs, imp, 2, false), (0x9B, tas, aby, 5, false),
            (0x9C, shy, abx, 5, false), (0x9D, sta, abx, 5, false), (0x9E, shx, aby, 5, false),
            (0x9F, ahx, aby, 5, false), (0xA0, ldy, imm, 2, false), (0xA1, lda, izx, 6, false),
            (0xA2, ldx, imm, 2, false), (0xA3, lax, izx, 6, false), (0xA4, ldy, zp0, 3, false),
            (0xA5, lda, zp0, 3, false), (0xA6, ldx, zp0, 3, false), (0xA7, lax, zp0, 3, false),
            (0xA8, tay, imp, 2, false), (0xA9, lda, imm, 2, false), (0xAA, tax, imp, 2, false),
            (0xAB, lax, imm, 2, false), (0xAC, ldy, abs, 4, false), (0xAD, lda, abs, 4, false),
            (0xAE, ldx, abs, 4, false), (0xAF, lax, abs, 4, false), (0xB0, bcs, rel, 2, true ),
            (0xB1, lda, izy, 5, true ), (0xB2, kil, imp, 2, false), (0xB3, lax, izy, 5, true ),
            (0xB4, ldy, zpx, 4, false), (0xB5, lda, zpx, 4, false), (0xB6, ldx, zpy, 4, false),
            (0xB7, lax, zpy, 4, false), (0xB8, clv, imp, 2, false), (0xB9, lda, aby, 4, true ),
            (0xBA, tsx, imp, 2, false), (0xBB, las, aby, 4, true ), (0xBC, ldy, abx, 4, true ),
            (0xBD, lda, abx, 4, true ), (0xBE, ldx, aby, 4, true ), (0xBF, lax, aby, 4, true ),
            (0xC0, cpy, imm, 2, false), (0xC1, cmp, izx, 6, false), (0xC2, nop, imm, 2, false),
            (0xC3, dcp, izx, 8, false), (0xC4, cpy, zp0, 3, false), (0xC5, cmp, zp0, 3, false),
            (0xC6, dec, zp0, 5, false), (0xC7, dcp, zp0, 5, false), (0xC8, iny, imp, 2, false),
            (0xC9, cmp, imm, 2, false), (0xCA, dex, imp, 2, false), (0xCB, axs, imm, 2, false),
            (0xCC, cpy, abs, 4, false), (0xCD, cmp, abs, 4, false), (0xCE, dec, abs, 6, false),
            (0xCF, dcp, abs, 6, false), (0xD0, bne, rel, 2, true ), (0xD1, cmp, izy, 5, true ),
            (0xD2, kil, imp, 2, false), (0xD3, dcp, izy, 8, false), (0xD4, nop, zpx, 4, false),
            (0xD5, cmp, zpx, 4, false), (0xD6, dec, zpx, 6, false), (0xD7, dcp, zpx, 6, false),
            (0xD8, cld, imp, 2, false), (0xD9, cmp, aby, 4, true ), (0xDA, nop, imp, 2, false),
            (0xDB, dcp, aby, 7, false), (0xDC, nop, abx, 4, true ), (0xDD, cmp, abx, 4, true ),
            (0xDE, dec, abx, 7, false), (0xDF, dcp, abx, 7, false), (0xE0, cpx, imm, 2, false),
            (0xE1, sbc, izx, 6, false), (0xE2, nop, imm, 2, false), (0xE3, isc, izx, 8, false),
            (0xE4, cpx, zp0, 3, false), (0xE5, sbc, zp0, 3, false), (0xE6, inc, zp0, 5, false),
            (0xE7, isc, zp0, 5, false), (0xE8, inx, imp, 2, false), (0xE9, sbc, imm, 2, false),
            (0xEA, nop, imp, 2, false), (0xEB, sbc, imm, 2, false), (0xEC, cpx, abs, 4, false),
            (0xED, sbc, abs, 4, false), (0xEE, inc, abs, 6, false), (0xEF, isc, abs, 6, false),
            (0xF0, beq, rel, 2, true ), (0xF1, sbc, izy, 5, true ), (0xF2, kil, imp, 2, false),
            (0xF3, isc, izy, 8, false), (0xF4, nop, zpx, 4, false), (0xF5, sbc, zpx, 4, false),
            (0xF6, inc, zpx, 6, false), (0xF7, isc, zpx, 6, false), (0xF8, sed, imp, 2, false),
            (0xF9, sbc, aby, 4, true ), (0xFA, nop, imp, 2, false), (0xFB, isc, aby, 7, false),
            (0xFC, nop, abx, 4, true ), (0xFD, sbc, abx, 4, true ), (0xFE, inc, abx, 7, false),
            (0xFF, isc, abx, 7, false),
        ]);
    }

    // Addressing modes

    /// Instructions using absolute addressing contain a full 16 bit address to
    /// identify the target location.
    fn abs(&mut self, _extra: bool) -> Address {
        let addr = self.loadw_bump_pc();
        Address::Absolute(addr)
    }

    /// The address to be accessed by an instruction using X register indexed
    /// absolute addressing is computed by taking the 16 bit address from the
    /// instruction and added the contents of the X register. For example if X
    /// contains $92 then an STA $2000,X instruction will store the accumulator
    /// at $2092 (e.g. $2000 + $92).
    fn abx(&mut self, extra: bool) -> Address {
        let val = self.loadw_bump_pc();
        let addr = val.wrapping_add(self.x as u16);

        // Use an extra cycle when crossing page boundaries
        let base_page = val & 0xFF00;
        let addr_page = addr & 0xFF00;
        if extra && base_page != addr_page {
            self.cy += 1;
        }

        Address::Absolute(addr)
    }

    /// The Y register indexed absolute addressing mode is the same as the
    /// previous mode only with the contents of the Y register added to the 16
    /// bit address from the instruction.
    fn aby(&mut self, extra: bool) -> Address {
        let val = self.loadw_bump_pc();
        let addr = val.wrapping_add(self.y as u16);

        // Use an extra cycle when crossing page boundaries
        let base_page = val & 0xFF00;
        let addr_page = addr & 0xFF00;
        if extra && base_page !=  addr_page {
            self.cy += 1;
        }

        Address::Absolute(addr)
    }

    /// Some instructions have an option to operate directly upon the
    /// accumulator. The programmer specifies this by using a special operand
    /// value, 'A'.
    fn acc(&mut self, _extra: bool) -> Address {
        Address::Accumulator
    }

    /// Immediate addressing allows the programmer to directly specify an 8 bit
    /// constant within the instruction. It is indicated by a '#' symbol
    /// followed by an numeric expression.
    fn imm(&mut self, _extra: bool) -> Address {
        let val = self.loadb_bump_pc();
        Address::Immediate(val)
    }

    /// For many 6502 instructions the source and destination of the information
    /// to be manipulated is implied directly by the function of the instruction
    /// itself and no further operand needs to be specified. Operations like
    /// 'Clear Carry Flag' (CLC) and 'Return from Subroutine' (RTS) are
    /// implicit.
    fn imp(&mut self, _extra: bool) -> Address {
        Address::Implied
    }

    /// JMP is the only 6502 instruction to support indirection. The instruction
    /// contains a 16 bit address which identifies the location of the least
    /// significant byte of another 16 bit memory address which is the real
    /// target of the instruction.
    ///
    /// For example if location $0120 contains $FC and location $0121 contains
    /// $BA then the instruction JMP ($0120) will cause the next instruction
    /// execution to occur at $BAFC (e.g. the contents of $0120 and $0121).
    fn ind(&mut self, _extra: bool) -> Address {
        let lo = self.loadb_bump_pc() as u16;
        let hi = self.loadb_bump_pc() as u16;

        Address::Indirect(word!(lo, hi))
    }

    /// Indexed indirect addressing is normally used in conjunction with a table
    /// of address held on zero page. The address of the table is taken from the
    /// instruction and the X register added to it (with zero page wrap around)
    /// to give the location of the least significant byte of the target
    /// address.
    fn izx(&mut self, _extra: bool) -> Address {
        let val = self.loadb_bump_pc();
        let addr = val.wrapping_add(self.x) as u16;

        if addr == 0x00FF {
            // Zero page access should wrap when reading from 0x00FF
            let addr = word!(self.loadb(0x00FF), self.loadb(0x0000));
            Address::Absolute(addr)
        } else {
            let addr = self.loadw(addr);
            Address::Absolute(addr)
        }
    }

    /// Indirect indexed addressing is the most common indirection mode used on
    /// the 6502. An instruction contains the zero page location of the least
    /// significant byte of 16 bit address. The Y register is dynamically added
    /// to this value to generated the actual target address for operation.
    fn izy(&mut self, extra: bool) -> Address {
        let val = self.loadb_bump_pc();

        let lo = self.loadb(val as u16);
        let hi = self.loadb(val.wrapping_add(1) as u16);
        let base = word!(lo, hi);
        let addr = base.wrapping_add(self.y as u16);

        // Use an extra cycle when crossing page boundaries
        let base_page = base & 0xFF00;
        let addr_page = addr & 0xFF00;
        if extra && base_page != addr_page {
            self.cy += 1;
        }

        Address::Absolute(addr)
    }

    /// Relative addressing mode is used by branch instructions (e.g. BEQ, BNE,
    /// etc.) which contain a signed 8 bit relative offset (e.g. -128 to +127)
    /// which is added to program counter if the condition is true. As the
    /// program counter itself is incremented during instruction execution by
    /// two the effective address range for the target instruction must be with
    /// -126 to +129 bytes of the branch.
    fn rel(&mut self, _extra: bool) -> Address {
        let delta: i32 = self.loadb_bump_pc().into();
        let addr = delta + self.pc as i32;
        Address::Absolute(addr as u16)
    }

    /// An instruction using zero page addressing mode has only an 8 bit address
    /// operand. This limits it to addressing only the first 256 bytes of memory
    /// (e.g. $0000 to $00FF) where the most significant byte of the address is
    /// always zero. In zero page mode only the least significant byte of the
    /// address is held in the instruction making it shorter by one byte
    /// (important for space saving) and one less memory fetch during execution
    /// (important for speed).
    ///
    /// An assembler will automatically select zero page addressing mode if the
    /// operand evaluates to a zero page address and the instruction supports
    /// the mode (not all do).
    fn zp0(&mut self, _extra: bool) -> Address {
        let addr = self.loadb_bump_pc().into();
        Address::Absolute(addr)
    }

    /// The address to be accessed by an instruction using indexed zero page
    /// addressing is calculated by taking the 8 bit zero page address from the
    /// instruction and adding the current value of the X register to it. For
    /// example if the X register contains $0F and the instruction LDA $80,X is
    /// executed then the accumulator will be loaded from $008F
    /// (e.g. $80 + $0F => $8F).
    ///
    /// The address calculation wraps around if the sum of the base address and
    /// the register exceed $FF. If we repeat the last example but with $FF in
    /// the X register then the accumulator will be loaded from $007F
    /// (e.g. $80 + $FF => $7F) and not $017F.
    fn zpx(&mut self, _extra: bool) -> Address {
        let val = self.loadb_bump_pc();
        let addr = val.wrapping_add(self.x).into();
        Address::Absolute(addr)
    }

    /// The address to be accessed by an instruction using indexed zero page
    /// addressing is calculated by taking the 8 bit zero page address from the
    /// instruction and adding the current value of the Y register to it. This
    /// mode can only be used with the LDX and STX instructions.
    fn zpy(&mut self, _extra: bool) -> Address {
        let val = self.loadb_bump_pc();
        let addr = val.wrapping_add(self.y).into();
        Address::Absolute(addr)
    }

    // Ops

    /// This instruction adds the contents of a memory location to the
    /// accumulator together with the carry bit. If overflow occurs the carry
    /// bit is set, this enables multiple byte addition to be performed.
    fn adc(&mut self, addr: Address) {
        let m: u16 = self.addr_loadb(&addr).into();
        let a: u16 = self.a.into();
        let c: u16 = self.flags.intersection(Flags::CARRY).bits().into();

        let result = a + m + c;

        let overflow = (!(a ^ m) & (a ^ result)) & 0x80 != 0;

        self.flags.set(Flags::OVERFLOW, overflow);
        self.flags.set(Flags::CARRY, result > 0x00FF);

        let result = result as u8;
        self.a = self.set_zn(result);
    }

    fn ahx(&mut self, _addr: Address) {
        panic!("AHX not implemented");
    }

    fn alr(&mut self, _addr: Address) {
        panic!("ALR not implemented");
    }

    fn anc(&mut self, _addr: Address) {
        panic!("ANC not implemented");
    }

    /// A logical AND is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    ///
    /// A,Z,N = A&M
    fn and(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.a = self.set_zn(val & self.a);
    }

    fn arr(&mut self, _addr: Address) {
        panic!("ARR not implemented");
    }

    /// This operation shifts all the bits of the accumulator or memory contents
    /// one bit left. Bit 0 is set to 0 and bit 7 is placed in the carry flag.
    /// The effect of this operation is to multiply the memory contents by 2
    /// (ignoring 2's complement considerations), setting the carry if the
    /// result will not fit in 8 bits.
    ///
    /// A,Z,C,N = M*2 or M,Z,C,N = M*2
    fn asl(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        let result = val.wrapping_shl(1);

        self.flags.set(Flags::CARRY, val & 0x80 == 0x80);
        self.set_zn(result);
        self.addr_storeb(&addr, result);
    }

    fn axs(&mut self, _addr: Address) {
        panic!("AXS not implemented");
    }

    /// If the carry flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bcc(&mut self, addr: Address) {
        let carry = self.flags.contains(Flags::CARRY);
        self.branch_base(!carry, addr);
    }

    /// If the carry flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bcs(&mut self, addr: Address) {
        let carry = self.flags.contains(Flags::CARRY);
        self.branch_base(carry, addr);
    }

    /// If the zero flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn beq(&mut self, addr: Address) {
        let zero = self.flags.contains(Flags::ZERO);
        self.branch_base(zero, addr);
    }

    /// This instruction is used to test if one or more bits are set in a target
    /// memory location. The mask pattern in A is ANDed with the value in memory
    /// to set or clear the zero flag, but the result is not kept.  Bits 7 and 6
    /// of the value from memory are copied into the N and V flags.
    ///
    /// A & M, N = M7, V = M6
    fn bit(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.flags.set(Flags::ZERO, self.a & val == 0);
        self.flags.set(Flags::OVERFLOW, val & 0x40 != 0);
        self.flags.set(Flags::NEGATIVE, val & 0x80 != 0);
    }

    /// If the negative flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bmi(&mut self, addr: Address) {
        let negative = self.flags.contains(Flags::NEGATIVE);
        self.branch_base(negative, addr);
    }

    /// If the zero flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bne(&mut self, addr: Address) {
        let zero = self.flags.contains(Flags::ZERO);
        self.branch_base(!zero, addr);
    }

    /// If the negative flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bpl(&mut self, addr: Address) {
        let negative = self.flags.contains(Flags::NEGATIVE);
        self.branch_base(!negative, addr);
    }

    /// The BRK instruction forces the generation of an interrupt request. The
    /// program counter and processor status are pushed on the stack then the
    /// IRQ interrupt vector at $FFFE/F is loaded into the PC and the break flag
    /// in the status set to one.
    fn brk(&mut self, _addr: Address) {
        let flags = (self.flags | Flags::from_bits_retain(0x30)).bits();
        // Push PC + 2 onto the stack. Since PC has already been incremented
        // from parsing the BRK, we only need to add one here.
        self.pushw(self.pc + 1);
        self.pushb(flags);
        self.pc = self.loadw(BRK_VECTOR);
    }

    /// If the overflow flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bvc(&mut self, addr: Address) {
        let overflow = self.flags.contains(Flags::OVERFLOW);
        self.branch_base(!overflow, addr);
    }

    /// If the overflow flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bvs(&mut self, addr: Address) {
        let overflow = self.flags.contains(Flags::OVERFLOW);
        self.branch_base(overflow, addr);
    }

    /// Set the carry flag to zero
    fn clc(&mut self, _addr: Address) {
        self.flags.remove(Flags::CARRY);
    }

    /// Sets the decimal mode flag to zero.
    fn cld(&mut self, _addr: Address) {
        self.flags.remove(Flags::DECIMAL);
    }

    /// Clears the interrupt disable flag allowing normal interrupt requests to
    /// be serviced.
    fn cli(&mut self, _addr: Address) {
        self.flags.remove(Flags::IRQ_DISABLE);
    }

    /// Clears the overflow flag.
    ///
    /// V = 0
    fn clv(&mut self, _addr: Address) {
        self.flags.remove(Flags::OVERFLOW);
    }

    /// This instruction compares the contents of the accumulator with another
    /// memory held value and sets the zero and carry flags as appropriate.
    ///
    /// Z,C,N = A-M
    fn cmp(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);

        self.flags.set(Flags::CARRY, self.a >= val);
        self.flags.set(Flags::ZERO, self.a == val);

        let result = self.a as i32 - val as i32;
        let negative = result & 0x80 != 0;
        self.flags.set(Flags::NEGATIVE, negative);
    }

    /// This instruction compares the contents of the X register with another
    /// memory held value and sets the zero and carry flags as appropriate.
    ///
    /// Z,C,N = X-M
    fn cpx(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);

        self.flags.set(Flags::CARRY, self.x >= val);
        self.flags.set(Flags::ZERO, self.x == val);

        let result = self.x as i16 - val as i16;
        let negative = result & 0x80 != 0;
        self.flags.set(Flags::NEGATIVE, negative);
    }

    /// This instruction compares the contents of the Y register with another
    /// memory held value and sets the zero and carry flags as appropriate.
    ///
    /// Z,C,N = Y-M
    fn cpy(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);

        self.flags.set(Flags::CARRY, self.y >= val);
        self.flags.set(Flags::ZERO, self.y == val);

        let result = self.y as i16 - val as i16;
        let negative = result & 0x80 != 0;
        self.flags.set(Flags::NEGATIVE, negative);
    }

    /// Assigns the result of decrementing the given address by one back to the
    /// source address, then subtracts the result from the accumulator setting
    /// the zero, carry, and negative flags as appropriate. This is the same as
    /// performing an DEC followed by a CMP.
    ///
    /// M = M - 1, Z,N,C = A - M
    fn dcp(&mut self, addr: Address) {
        self.dec(addr);
        self.cmp(addr);
    }

    /// Subtracts one from the value held at a specified memory location setting
    /// the zero and negative flags as appropriate.
    ///
    /// M,Z,N = M-1
    fn dec(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr).wrapping_sub(1);
        self.addr_storeb(&addr, val);
        self.set_zn(val);
    }

    /// Subtracts one from the X register setting the zero and negative flags as
    /// appropriate.
    ///
    /// X,Z,N = X-1
    fn dex(&mut self, _addr: Address) {
        self.x = self.set_zn(self.x.wrapping_sub(1));
    }

    /// Subtracts one from the X register setting the zero and negative flags as
    /// appropriate.
    fn dey(&mut self, _addr: Address) {
        self.y = self.set_zn(self.y.wrapping_sub(1));
    }

    /// An exclusive OR is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    ///
    /// A,Z,N = A^M
    fn eor(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.a ^= val;
        self.set_zn(self.a);
    }

    /// Adds one to the value held at a specified memory location setting the
    /// zero and negative flags as appropriate.
    ///
    /// M,Z,N = M+1
    fn inc(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr).wrapping_add(1);
        self.addr_storeb(&addr, val);
        self.set_zn(val);
    }

    /// Adds one to the X register setting the zero and negative flags as
    /// appropriate.
    ///
    /// X,Z,N = X+1
    fn inx(&mut self, _addr: Address) {
        self.x = self.set_zn(self.x.wrapping_add(1));
    }

    /// Adds one to the Y register setting the zero and negative flags as
    /// appropriate.
    ///
    /// Y,Z,N = Y+1
    fn iny(&mut self, _addr: Address) {
        self.y = self.set_zn(self.y.wrapping_add(1));
    }

    /// Assigns the result of incrementing the given address by one back to the
    /// source address, then subtracts the result from the accumulator setting
    /// the zero, carry, and negative flags as appropriate. This is the same as
    /// performing an INC followed by a CMP.
    ///
    /// M = M + 1, Z,N,C = A - M
    fn isc(&mut self, addr: Address) {
        self.inc(addr);
        self.sbc(addr);
    }

    /// Sets the program counter to the address specified by the operand.
    fn jmp(&mut self, addr: Address) {
        match addr {
            Address::Indirect(addr) => {
                // 6502 bug: when the indirect address falls on a page boundary
                // the address is fetched by wrapping to 0 of the same page, not
                // by reading the high byte from the next page as expected
                if addr & 0x00FF == 0x00FF {
                    let page = addr & 0xFF00;
                    let lo = self.loadb(page + 0x00FF);
                    let hi = self.loadb(page);
                    self.pc = word!(lo, hi);
                } else {
                    self.pc = self.loadw(addr);
                }
            }
            Address::Absolute(addr) => {
                self.pc = addr;
            }
            _ => unreachable!("JMP only supports absolute or indirect addresses"),
        }
    }

    /// The JSR instruction pushes the address (minus one) of the return point
    /// on to the stack and then sets the program counter to the target memory
    /// address.
    fn jsr(&mut self, addr: Address) {
        if let Address::Absolute(addr) = addr {
            // PC will be pointing at the first byte of the next instruction, but
            // JSR should store a stack pointer that points to the last byte of the
            // JSR instruction itself.
            let return_addr = self.pc.wrapping_sub(1);
            self.pushw(return_addr);
            self.pc = addr;
            return;
        }

        unreachable!("Only absolute addresses are supported for JSR");
    }

    /// Halts the CPU.
    fn kil(&mut self, _addr: Address) {
        panic!("CPU KIL");
    }

    fn las(&mut self, _addr: Address) {
        panic!("LAS not implemented");
    }

    /// Loads a byte of memory into both the accumulator and the x register
    /// setting the zero and negative flags as appropriate
    ///
    /// A,X,Z,N = M
    fn lax(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.a = val;
        self.x = val;
        self.set_zn(val);
    }

    /// Loads a byte of memory into the accumulator setting the zero and
    /// negative flags as appropriate.
    ///
    /// A,Z,N = M
    fn lda(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.a = self.set_zn(val);
    }

    /// Loads a byte of memory into the X register setting the zero and negative
    /// flags as appropriate.
    ///
    /// X,Z,N = M
    fn ldx(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.x = self.set_zn(val);
    }

    /// Loads a byte of memory into the Y register setting the zero and negative
    /// flags as appropriate.
    ///
    /// Y,Z,N = M
    fn ldy(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.y = self.set_zn(val);
    }

    /// Each of the bits in A or M is shift one place to the right. The bit that
    /// was in bit 0 is shifted into the carry flag. Bit 7 is set to zero.
    ///
    /// A,C,Z,N = A/2 or M,C,Z,N = M/2
    fn lsr(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        let result = val >> 1;

        self.flags.set(Flags::CARRY, val & 0x01 != 0);
        self.set_zn(result);
        self.addr_storeb(&addr, result);
    }

    /// The NOP instruction causes no changes to the processor other than the
    /// normal incrementing of the program counter to the next instruction.
    fn nop(&mut self, _addr: Address) {}

    /// An inclusive OR is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    ///
    /// A,Z,N = A | M
    fn ora(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        self.a = self.set_zn(self.a | val);
    }

    /// Pushes a copy of the accumulator on to the stack.
    fn pha(&mut self, _addr: Address) {
        self.pushb(self.a);
    }

    /// Pushes a copy of the status flags on to the stack.
    fn php(&mut self, _addr: Address) {
        // When pushing, bits 4 & 5 are always 1.
        let bits = (self.flags | Flags::from_bits_retain(0x30)).bits();
        self.pushb(bits);
    }

    /// Pulls an 8 bit value from the stack and into the accumulator. The zero
    /// and negative flags are set as appropriate.
    fn pla(&mut self, _addr: Address) {
        self.a = self.popb();
        self.set_zn(self.a);
    }

    /// Pulls an 8 bit value from the stack and into the processor flags. The
    /// flags will take on new states as determined by the value pulled.
    fn plp(&mut self, _addr: Address) {
        // the 4th bit ("B flag") isn't set by this op
        // the 5th bit isn't a bit, but for emulation it's just always high.
        let flags = self.popb() & 0xEF | 0x20;
        self.flags = Flags::from_bits_retain(flags);
    }

    fn rla(&mut self, addr: Address) {
        self.rol(addr);
        self.and(addr);
    }

    /// Move each of the bits in either A or M one place to the left. Bit 0 is
    /// filled with the current value of the carry flag whilst the old bit 7
    /// becomes the new carry flag value.
    fn rol(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        let mut result = val.wrapping_shl(1);

        if self.flags.intersects(Flags::CARRY) {
            result += 1;
        }

        self.addr_storeb(&addr, result);

        self.flags.set(Flags::CARRY, val & 0x80 != 0);
        self.flags.set(Flags::NEGATIVE, result & 0x80 != 0);
        self.flags.set(Flags::ZERO, self.a == 0);
    }

    /// Move each of the bits in either A or M one place to the right. Bit 7 is
    /// filled with the current value of the carry flag whilst the old bit 0
    /// becomes the new carry flag value.
    fn ror(&mut self, addr: Address) {
        let val = self.addr_loadb(&addr);
        let mut result = val >> 1;

        if self.flags.intersects(Flags::CARRY) {
            result |= 0x80;
        }

        self.flags.set(Flags::CARRY, val & 0x01 != 0);
        self.set_zn(result);

        self.addr_storeb(&addr, result);
    }

    fn rra(&mut self, addr: Address) {
        self.ror(addr);
        self.adc(addr);
    }

    /// The RTI instruction is used at the end of an interrupt processing
    /// routine. It pulls the processor flags from the stack followed by the
    /// program counter.
    fn rti(&mut self, _addr: Address) {
        // Bit 5 isn't real, but it's always high for emu purposes
        self.flags = Flags::from_bits_retain(self.popb() | 0x20);
        self.pc = self.popw();
    }

    /// The RTS instruction is used at the end of a subroutine to return to the
    /// calling routine. It pulls the program counter (minus one) from the
    /// stack.
    fn rts(&mut self, _addr: Address) {
        self.pc = self.popw().wrapping_add(1);
    }

    /// The SAX instruction stores the binary AND of the accumulator and the X
    /// register into the given memory location.
    ///
    /// M = S & X
    fn sax(&mut self, addr: Address) {
        self.addr_storeb(&addr, self.a & self.x);
    }

    /// This instruction subtracts the contents of a memory location to the
    /// accumulator together with the not of the carry bit. If overflow occurs
    /// the carry bit is clear, this enables multiple byte subtraction to be
    /// performed.
    ///
    /// A,Z,C,N = A-M-(1-C)
    fn sbc(&mut self, addr: Address) {
        let mut m: u16 = self.addr_loadb(&addr).into();
        let a: u16 = self.a.into();
        // 1 when carry flag is set, 0 otherwise
        let c: u16 = self.flags.intersection(Flags::CARRY).bits().into();

        m ^= 0x00FF;
        let result = a + m + c;

        // Set the carry flag if the (signed) result >= 0
        self.flags.set(Flags::CARRY, result & 0xFF00 != 0);

        // Set the overflow flag if the sign flipped
        let overflow = (result ^ a) & (result ^ m) & 0x80 != 0;
        self.flags.set(Flags::OVERFLOW, overflow);

        self.a = self.set_zn(result as u8);
    }

    /// Set the carry flag to one
    fn sec(&mut self, _addr: Address) {
        self.flags.insert(Flags::CARRY);
    }

    /// Set the decimal mode flag to one.
    ///
    /// D = 1
    fn sed(&mut self, _addr: Address) {
        self.flags.insert(Flags::DECIMAL);
    }

    /// Set the interrupt disable flag to one
    fn sei(&mut self, _addr: Address) {
        self.flags.insert(Flags::IRQ_DISABLE);
    }

    fn shx(&mut self, _addr: Address) {
        panic!("SHX not implemented");
    }

    fn shy(&mut self, _addr: Address) {
        panic!("SHY not implemented");
    }

    /// Assigns the result of shifting bits to the left once from the given
    /// address back t othat address, and then assigns A | M back to the
    /// accumulator. Setting the negative, zero, and carry flags as appropriate.
    /// This is the same as performing an ASL followed by an ORA.
    ///
    /// M = M << 1, N,Z,C,A = A | M
    fn slo(&mut self, addr: Address) {
        self.asl(addr);
        self.ora(addr);
    }

    fn sre(&mut self, addr: Address) {
        self.lsr(addr);
        self.eor(addr);
    }

    /// Stores the contents of the accumulator into memory.
    ///
    /// M = A
    fn sta(&mut self, addr: Address) {
        self.addr_storeb(&addr, self.a);
    }

    /// Stores the contents of the X register into memory.
    fn stx(&mut self, addr: Address) {
        self.addr_storeb(&addr, self.x);
    }

    /// Stores the contents of the Y register into memory.
    fn sty(&mut self, addr: Address) {
        self.addr_storeb(&addr, self.y);
    }

    fn tas(&mut self, _addr: Address) {
        panic!("TAS not implemented");
    }

    /// Copies the current contents of the accumulator into the X register and
    /// sets the zero and negative flags as appropriate.
    ///
    /// X = A
    fn tax(&mut self, _addr: Address) {
        self.x = self.set_zn(self.a);
    }

    /// Copies the current contents of the accumulator into the Y register and
    /// sets the zero and negative flags as appropriate.
    ///
    /// Y = A
    fn tay(&mut self, _addr: Address) {
        self.y = self.set_zn(self.a);
    }

    /// Copies the current contents of the stack register into the X register
    /// and sets the zero and negative flags as appropriate.
    ///
    /// X = S
    fn tsx(&mut self, _addr: Address) {
        self.x = self.set_zn(self.s);
    }

    /// Copies the current contents of the X register into the accumulator and
    /// sets the zero and negative flags as appropriate.
    fn txa(&mut self, _addr: Address) {
        self.a = self.set_zn(self.x);
    }

    /// Copies the current contents of the X register into the stack register.
    fn txs(&mut self, _addr: Address) {
        self.s = self.x;
    }

    /// Copies the current contents of the Y register into the accumulator and
    /// sets the zero and negative flags as appropriate.
    fn tya(&mut self, _addr: Address) {
        self.a = self.set_zn(self.y);
    }

    fn xaa(&mut self, _addr: Address) {
        panic!("XAA not implemented");
    }

    // Branch helpers

    fn branch_base(&mut self, condition: bool, addr: Address) {
        if !condition {
            return;
        }

        if let Address::Absolute(addr) = addr {
            self.cy += 1;

            // When crossing page boundaries yet another cycle is needed
            if self.pc & 0xFF00 != addr & 0xFF00 {
                self.cy += 1;
            }

            self.pc = addr;
            return;
        }

        unreachable!("Invalid address in branch instruction");
    }

    // Addressing helpers

    fn addr_loadb(&mut self, addr: &Address) -> u8 {
        match addr {
            Address::Absolute(addr) => self.loadb(*addr),
            Address::Accumulator    => self.a,
            Address::Immediate(val) => *val,
            Address::Indirect(_)    => unreachable!("Can't load from indirect address"),
            Address::Implied        => unreachable!("Can't load from implied address"),
        }
    }

    fn addr_storeb(&mut self, addr: &Address, val: u8) {
        match addr {
            Address::Absolute(addr) => self.storeb(*addr, val),
            Address::Accumulator => self.a = val,
            _ => panic!("Can't store to this address type"),
        }
    }

    // PC helpers

    fn loadb_bump_pc(&mut self) -> u8 {
        let val = self.loadb(self.pc);
        self.pc += 1;
        val
    }

    fn loadw_bump_pc(&mut self) -> u16 {
        let val = self.loadw(self.pc);
        self.pc += 2;
        val
    }

    // flags

    fn set_zn(&mut self, val: u8) -> u8 {
        self.flags.set(Flags::ZERO, val == 0);
        self.flags.set(Flags::NEGATIVE, (val & 0x80) != 0);
        val
    }

    // Stack helpers

    fn pushb(&mut self, val: u8) {
        let s: u16 = self.s.into();
        let addr = STACK_BASE + s;
        self.s -= 1;
        self.storeb(addr, val);
    }

    fn pushw(&mut self, val: u16) {
        let hi = val >> 8;
        let lo = val & 0xFF;

        self.pushb(hi as u8);
        self.pushb(lo as u8);
    }

    fn popb(&mut self) -> u8 {
        let s = self.s as u16;
        let addr = STACK_BASE + s + 1;
        self.s += 1;
        self.loadb(addr)
    }

    fn popw(&mut self) -> u16 {
        let s = self.s as u16;
        let addr = STACK_BASE + s + 1;
        self.s += 2;
        self.loadw(addr)
    }
}

impl<M: Mem> Mem for Cpu<M> {
    fn peekb(&self, addr: u16) -> u8 {
        self.mem.peekb(addr)
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        self.mem.loadb(addr)
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        if addr == 0x4014 {
            self.dma(val);
        } else {
            self.mem.storeb(addr, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::assert_matches;

    macro_rules! cpu {
        ( ) => { cpu!(0x00) };

        ( $flags:expr ) => { cpu!($flags, vec![]) };

        ( $flags:expr, $mem:expr ) => {{
            let mut cpu = Cpu::new(VecMem { mem: $mem });
            cpu.pc = 0;
            cpu.s = 0xFF;
            cpu.flags = Flags::from_bits_retain($flags);
            cpu
        }};
    }

    struct VecMem {
        mem: Vec<u8>,
    }

    impl Mem for VecMem {
        fn peekb(&self, addr: u16) -> u8 {
            self.mem[addr as usize]
        }

        fn storeb(&mut self, addr: u16, val: u8) {
            self.mem[addr as usize] = val;
        }
    }

    mod interrupts {
        use super::*;

        #[test]
        fn reset_sets_i_flag() {
            let mut cpu = cpu!(0x00, vec![0; 0x10000]);
            cpu.reset();
            assert!(cpu.flags.contains(Flags::IRQ_DISABLE));
        }

        #[test]
        fn nmi_sets_i_flag() {
            let mut cpu = cpu!(0x00, vec![0; 0x10000]);
            cpu.nmi();
            assert!(cpu.flags.contains(Flags::IRQ_DISABLE));
        }

        #[test]
        fn nmi_clears_b_flag() {
            let mut cpu = cpu!(Flags::BREAK.bits(), vec![0; 0x10000]);
            cpu.nmi();
            assert!(!cpu.flags.contains(Flags::BREAK));
        }

        #[test]
        fn nmi_pushes_pc_to_stack() {
            let mut cpu = cpu!(0x00, vec![0; 0x10000]);
            cpu.s = 0xFF;
            cpu.pc = 0xABCD;
            cpu.nmi();
            assert_eq!(cpu.mem.mem[0x01FE], 0xCD);
            assert_eq!(cpu.mem.mem[0x01FF], 0xAB);
        }

        #[test]
        fn nmi_pushes_flags_to_stack() {
            let mut cpu = cpu!(Flags::ZERO.bits() | Flags::CARRY.bits(), vec![0; 0x10000]);
            cpu.s = 0xFF;
            cpu.nmi();
            assert_eq!(cpu.mem.mem[0x01FD], Flags::ZERO.bits() | Flags::CARRY.bits());
        }

        #[test]
        fn nmi_sets_pc() {
            let mut mem = vec![0; 0x10000];
            mem[0xFFFA] = 0xCD;
            mem[0xFFFB] = 0xAB;
            let mut cpu = cpu!(0x00, mem);
            cpu.nmi();
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn irq_ignored_with_i_flag() {
            let mut cpu = cpu!(Flags::IRQ_DISABLE.bits(), vec![0; 0x10000]);
            cpu.pc = 0xABCD;
            cpu.irq();
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn irq_sets_irq_flag() {
            let mut cpu = cpu!(0x00, vec![0; 0x10000]);
            cpu.irq();
            assert!(cpu.flags.intersects(Flags::IRQ_DISABLE));
        }

        #[test]
        fn irq_clears_b_flag() {
            let mut cpu = cpu!(Flags::BREAK.bits(), vec![0; 0x10000]);
            cpu.irq();
            assert!(!cpu.flags.intersects(Flags::BREAK));
        }

        #[test]
        fn irq_pushes_pc_to_stack() {
            let mut cpu = cpu!(0x00, vec![0x00; 0x10000]);
            cpu.s = 0xFF;
            cpu.pc = 0xABCD;
            cpu.irq();
            assert_eq!(cpu.mem.mem[0x01FE], 0xCD);
            assert_eq!(cpu.mem.mem[0x01FF], 0xAB);
        }

        #[test]
        fn irq_pushes_flags_to_stack() {
            let mut cpu = cpu!(Flags::ZERO.bits() | Flags::CARRY.bits(), vec![0; 0x10000]);
            cpu.irq();
            assert_eq!(cpu.mem.mem[0x01FD], Flags::ZERO.bits() | Flags::CARRY.bits());
        }

        #[test]
        fn irq_sets_pc() {
            let mut mem = vec![0; 0x10000];
            mem[0xFFFE] = 0xCD;
            mem[0xFFFF] = 0xAB;
            let mut cpu = cpu!(0x00, mem);
            cpu.irq();
            assert_eq!(cpu.pc, 0xABCD);
        }
    }

    mod adr {
        use super::*;

        #[test]
        fn abs_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0; 10]);
            cpu.abs(false);
            assert_eq!(cpu.pc, 2);
        }

        #[test]
        fn abs_loads_from_pc() {
            let mut cpu = cpu!(0x00, vec![0xCD, 0xAB]);
            let addr = cpu.abs(false);
            assert_matches!(addr, Address::Absolute(0xABCD));
        }

        #[test]
        fn abx_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x00]);
            cpu.abx(false);
            assert_eq!(cpu.pc, 2);
        }

        #[test]
        fn abx_loads_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x80, 0x00]);
            let addr = cpu.abx(false);
            assert_matches!(addr, Address::Absolute(0x80));
        }

        #[test]
        fn abx_adds_x_to_addr() {
            let mut cpu = cpu!(0x00, vec![0x80, 0x80]);
            cpu.x = 0x08;
            let addr = cpu.abx(false);
            assert_matches!(addr, Address::Absolute(0x8088));
        }

        #[test]
        fn abx_wraps_word() {
            let mut cpu = cpu!(0x00, vec![0xFF, 0xFF]);
            cpu.x = 0x01;
            let addr = cpu.abx(false);
            assert_matches!(addr, Address::Absolute(0x00));
        }

        #[test]
        fn abx_uses_extra_cycle_across_pages() {
            let mut cpu = cpu!(0x00, vec![0xFF, 0x00]);
            cpu.x = 0x01;
            cpu.abx(true); // 0x00FF -> 0x0100
            assert_eq!(cpu.cy, 1);
        }

        #[test]
        fn abx_uses_no_cycle_in_same_page() {
            let mut cpu = cpu!(0x00, vec![0xC0, 0x00]);
            cpu.x = 0x0C;
            cpu.abx(true);
            assert_eq!(cpu.cy, 0);
        }

        #[test]
        fn aby_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x00]);
            cpu.aby(false);
            assert_eq!(cpu.pc, 2);
        }

        #[test]
        fn aby_loads_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x80, 0x70]);
            let addr = cpu.aby(false);
            assert_matches!(addr, Address::Absolute(0x7080));
        }

        #[test]
        fn aby_adds_y_to_addr() {
            let mut cpu = cpu!(0x00, vec![0xC0, 0xF0]);
            cpu.y = 0x0C;
            let addr = cpu.aby(false);
            assert_matches!(addr, Address::Absolute(0xF0CC));
        }

        #[test]
        fn aby_wraps_word() {
            let mut cpu = cpu!(0x00, vec![0xFF, 0xFF]);
            cpu.y = 0x02;
            let addr = cpu.aby(false);
            assert_matches!(addr, Address::Absolute(0x0001));
        }

        #[test]
        fn aby_uses_extra_cycle_across_pages() {
            let mut cpu = cpu!(0x00, vec![0xFF, 0x0C]);
            cpu.y = 0xFF;
            cpu.aby(true);
            assert_eq!(cpu.cy, 1);
        }

        #[test]
        fn aby_uses_no_cycle_in_same_page() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x0C]);
            cpu.y = 0xFF;
            cpu.aby(true);
            assert_eq!(cpu.cy, 0);
        }

        #[test]
        fn imm_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x00]);
            cpu.imm(false);
            assert_eq!(cpu.pc, 0x0001);
        }

        #[test]
        fn imm_loads_byte_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x01]);
            let addr = cpu.imm(false);
            assert_matches!(addr, Address::Immediate(0x01));
        }

        #[test]
        fn ind_incrments_pc() {
            let mut cpu = cpu!(0x00, vec![0; 4]);
            cpu.ind(false);
            assert_matches!(cpu.pc, 2);
        }

        #[test]
        fn ind_loads_byte_from_mem() {
            let mut cpu = cpu!(0x00, vec![0x02, 0x00]);
            let addr = cpu.ind(false);
            assert_matches!(addr, Address::Indirect(0x02));
        }

        #[test]
        fn izx_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0; 2]);
            cpu.izx(false);
            assert_eq!(cpu.pc, 1);
        }

        #[test]
        fn izx_loads_word_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x01, 0x01, 0x02]);
            let addr = cpu.izx(false);
            assert_matches!(addr, Address::Absolute(0x0201));
        }

        #[test]
        fn izx_adds_x_to_addr() {
            let mut cpu = cpu!(0x00, vec![0x01, 0x02, 0x03, 0x04]);
            cpu.x = 0x01;
            let addr = cpu.izx(false);
            assert_matches!(addr, Address::Absolute(0x0403));
        }

        #[test]
        fn izx_wraps_x_and_byte() {
            let mut cpu = cpu!(0x00, vec![0x01, 0x02]);
            cpu.x = 0xFF;
            let addr = cpu.izx(false);
            assert_matches!(addr, Address::Absolute(0x0201));
        }

        #[test]
        fn izy_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x01, 0x02, 0x03]);
            cpu.izy(false);
            assert_eq!(cpu.pc, 1);
        }

        #[test]
        fn izy_loads_word_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x01, 0x02, 0x03]);
            let addr = cpu.izy(false);
            assert_matches!(addr, Address::Absolute(0x0302));
        }

        #[test]
        fn izy_adds_y_to_addr() {
            let mut cpu = cpu!(0x00, vec![0x01, 0x02, 0x03]);
            cpu.y = 0x10;
            let addr = cpu.izy(false);
            assert_matches!(addr, Address::Absolute(0x0312));
        }

        #[test]
        fn izy_wraps_y_and_byte() {
            let mut cpu = cpu!(0x00, vec![0x01, 0xFF, 0xFF]);
            cpu.y = 0x01;
            let addr = cpu.izy(false);
            assert_matches!(addr, Address::Absolute(0x00));
        }

        #[test]
        fn izy_adds_cycle_when_crossing_pages() {
            let mut cpu = cpu!(0x00, vec![0x01, 0xFF, 0xFF]);
            cpu.y = 0x01;
            cpu.izy(true);
            assert_eq!(cpu.cy, 1);
        }

        #[test]
        fn izy_doesnt_add_cycle_in_same_page() {
            let mut cpu = cpu!(0x00, vec![0x01, 0xFE, 0xFF]);
            cpu.y = 0x01;
            cpu.izy(true);
            assert_eq!(cpu.cy, 0);
        }

        #[test]
        fn rel_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0; 2]);
            cpu.rel(false);
            assert_eq!(cpu.pc, 1);
        }

        #[test]
        fn rel_creates_positive_delta() {
            let mut cpu = cpu!(0x00, vec![0x01; 6]);
            cpu.pc = 5;
            let addr = cpu.rel(false);
            assert_matches!(addr, Address::Absolute(0x07));
        }

        #[test]
        fn rel_creates_negative_delta() {
            let mut cpu = cpu!(0x00, vec![0xFE; 6]);
            cpu.pc = 0x0005;
            let addr = cpu.rel(false);
            assert_matches!(addr, Address::Absolute(0x0004));
        }

        #[test]
        fn zp0_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x00]);
            cpu.zp0(false);
            assert_eq!(cpu.pc, 1);
        }

        #[test]
        fn zp0_loads_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x10]);
            let addr = cpu.zp0(false);
            assert_matches!(addr, Address::Absolute(0x0010));
        }

        #[test]
        fn zpx_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x00]);
            cpu.zpx(false);
            assert_eq!(cpu.pc, 1);
        }

        #[test]
        fn zpx_loads_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x03]);
            let addr = cpu.zpx(false);
            assert_matches!(addr, Address::Absolute(0x0003));
        }

        #[test]
        fn zpx_adds_x_to_addr() {
            let mut cpu = cpu!(0x00, vec![0x03]);
            cpu.x = 0x02;
            let addr = cpu.zpx(false);
            assert_matches!(addr, Address::Absolute(0x0005));
        }

        #[test]
        fn zpx_wraps_overflows() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.x = 0x02;
            let addr = cpu.zpx(false);
            assert_matches!(addr, Address::Absolute(0x0001));
        }

        #[test]
        fn zpy_increments_pc() {
            let mut cpu = cpu!(0x00, vec![0x00]);
            cpu.zpy(false);
            assert_eq!(cpu.pc, 1);
        }

        #[test]
        fn zpy_loads_from_pc() {
            let mut cpu = cpu!(0x00, vec![0x01]);
            let addr = cpu.zpy(false);
            assert_matches!(addr, Address::Absolute(0x0001));
        }

        #[test]
        fn zpy_adds_y_to_addr() {
            let mut cpu = cpu!(0x00, vec![0x01]);
            cpu.y = 0x02;
            let addr = cpu.zpy(false);
            assert_matches!(addr, Address::Absolute(0x0003));
        }

        #[test]
        fn zpy_wraps_overflows() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.y = 0x02;
            let addr = cpu.zpy(false);
            assert_matches!(addr, Address::Absolute(0x0001));
        }
    }

    mod ops {
        use super::*;

        #[test]
        fn adc_adds_mem() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.adc(Address::Immediate(0x01));
            assert_eq!(cpu.a, 0x01);
        }

        #[test]
        fn adc_adds_carry() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.flags.insert(Flags::CARRY);
            cpu.adc(Address::Immediate(0x01));
            assert_eq!(cpu.a, 0x02);
        }

        #[test]
        fn adc_wraps_byte_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0xFF;
            cpu.adc(Address::Immediate(0xFF));
            assert_eq!(cpu.a, 0xFE);
        }

        #[test]
        fn adc_sets_v_for_underflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x80;
            cpu.adc(Address::Immediate(0xFF));
            assert!(cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn adc_sets_v_for_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x7F;
            cpu.adc(Address::Immediate(0x01));
            assert!(cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn adc_clears_v_for_non_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.flags.insert(Flags::OVERFLOW);
            cpu.a = 0x05;
            cpu.adc(Address::Immediate(0x05));
            assert!(!cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn adc_sets_c_for_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0xFF;
            cpu.adc(Address::Immediate(0x01));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn adc_clears_c_no_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.flags.insert(Flags::CARRY);
            cpu.a = 0x01;
            cpu.adc(Address::Immediate(0x01));
            assert!(!cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn and_logical_ands_a() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0b10101010;
            cpu.and(Address::Immediate(0b10000010));
            assert_eq!(cpu.a, 0b10000010);
        }

        #[test]
        fn and_sets_zero_flag() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.and(Address::Immediate(0xFF));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn and_sets_negative_flag() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x80;
            cpu.and(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn asl_shifts_left() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x40;
            cpu.asl(Address::Accumulator);
            assert_eq!(0x80, cpu.a);
        }

        #[test]
        fn asl_sets_carry_on_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x80;
            cpu.asl(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn asl_doesnt_set_carry_flag_without_overflow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x40;
            cpu.asl(Address::Accumulator);
            assert!(!cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn asl_sets_negative_bit() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0xF7;
            cpu.asl(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn asl_sets_zero_bit() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x80;
            cpu.asl(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn bcc_jumps_when_carry_is_clear() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.bcc(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bcc_doesnt_jump_when_carry_is_set() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.flags.insert(Flags::CARRY);
            cpu.bcc(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn bcs_jumps_when_carry_is_set() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.flags.insert(Flags::CARRY);
            cpu.bcs(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bcs_doesnt_jump_when_carry_is_clear() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.bcs(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn beq_jumps_when_zero_is_set() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.flags.insert(Flags::ZERO);
            cpu.beq(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn beq_doesnt_jump_when_zero_is_clear() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.beq(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn bit_sets_n_with_test_value_bit_7() {
            let mut cpu = cpu!(0x00, vec![0x80]);
            cpu.bit(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn bit_sets_v_with_test_value_bit_6() {
            let mut cpu = cpu!(0x00, vec![0x40]);
            cpu.bit(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn bit_sets_z_when_test_and_acc_is_zero() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.bit(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn bit_doesnt_set_z_when_test_and_acc_isnt_zero() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.a = 0x80;
            cpu.bit(Address::Absolute(0x00));
            assert!(!cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn bit_desnt_affect_a() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.a = 0x80;
            cpu.bit(Address::Absolute(0x00));
            assert_eq!(cpu.a, 0x80);
        }

        #[test]
        fn bmi_jumps_when_negative_set() {
            let mut cpu = cpu!(Flags::NEGATIVE.bits());
            cpu.bmi(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bmi_doesnt_jump_when_negative_not_set() {
            let mut cpu = cpu!();
            cpu.bmi(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn bne_jumps_when_zero_clear() {
            let mut cpu = cpu!();
            cpu.bne(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bne_doesnt_jump_when_zero_set() {
            let mut cpu = cpu!(Flags::ZERO.bits());
            cpu.bne(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn bpl_jumps_when_negative_clear() {
            let mut cpu = cpu!();
            cpu.bpl(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bpl_doesnt_jump_when_negative_set() {
            let mut cpu = cpu!(Flags::NEGATIVE.bits());
            cpu.bpl(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn brk_pushes_pc_to_stack() {
            let mut cpu = cpu!(0x00, vec![0; 0x10000]);
            cpu.pc = 0x0102;
            cpu.brk(Address::Implied);
            assert_eq!(cpu.mem.mem[0x01FE], 0x03);
            assert_eq!(cpu.mem.mem[0x01FF], 0x01);
        }

        #[test]
        fn brk_pushes_flags_to_stack() {
            let flags = 0x02;
            let mut cpu = cpu!(flags, vec![0; 0x10000]);
            cpu.brk(Address::Implied);
            assert!(cpu.mem.mem[0x01FD] & flags == flags);
        }

        #[test]
        fn brk_sets_break_bit() {
            let mut cpu = cpu!(0x00, vec![0; 0x10000]);
            cpu.brk(Address::Implied);
            assert_eq!(cpu.mem.mem[0x01FD] & Flags::BREAK.bits(), Flags::BREAK.bits());
        }

        #[test]
        fn brk_loads_pc_from_fffe() {
            let mut mem = vec![0; 0x10000];
            mem[0xFFFE] = 0xCD;
            mem[0xFFFF] = 0xAB;

            let mut cpu = cpu!(0x00, mem);
            cpu.brk(Address::Implied);
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bvc_jumps_when_overflow_clear() {
            let mut cpu = cpu!();
            cpu.bvc(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bvc_doesnt_jump_when_overflow_set() {
            let mut cpu = cpu!(Flags::OVERFLOW.bits());
            cpu.bvc(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn bvs_jumps_when_overflow_set() {
            let mut cpu = cpu!(Flags::OVERFLOW.bits());
            cpu.bvs(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn bvs_doesnt_jump_when_overflow_clear() {
            let mut cpu = cpu!();
            cpu.bvs(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0);
        }

        #[test]
        fn clc_clears_carry_flag() {
            let mut cpu = cpu!(Flags::CARRY.bits());
            cpu.clc(Address::Implied);
            assert!(!cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cld_clears_decimal_flag() {
            let mut cpu = cpu!(Flags::DECIMAL.bits());
            cpu.cld(Address::Implied);
            assert!(!cpu.flags.intersects(Flags::DECIMAL));
        }

        #[test]
        fn cli_clears_interrupt_disable_flag() {
            let mut cpu = cpu!(Flags::IRQ_DISABLE.bits());
            cpu.cli(Address::Implied);
            assert!(!cpu.flags.intersects(Flags::IRQ_DISABLE));
        }

        #[test]
        fn clv_clears_overflow_flag() {
            let mut cpu = cpu!(Flags::OVERFLOW.bits());
            cpu.clv(Address::Implied);
            assert!(!cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn cmp_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x80;
            cpu.cmp(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn cmp_sets_carry_when_mem_lt_acc() {
            let mut cpu = cpu!();
            cpu.a = 0xFF;
            cpu.cmp(Address::Immediate(0x01));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cmp_sets_carry_when_mem_eq_acc() {
            let mut cpu = cpu!();
            cpu.a = 0xFF;
            cpu.cmp(Address::Immediate(0xFF));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cmp_sets_negative_with_bit7_result() {
            let mut cpu = cpu!();
            cpu.a = 0xFF;
            cpu.cmp(Address::Immediate(0x1F));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn cpx_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.x = 0x80;
            cpu.cpx(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn cpx_sets_carry_when_mem_lt_x() {
            let mut cpu = cpu!();
            cpu.x = 0xFF;
            cpu.cpx(Address::Immediate(0x01));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cpx_sets_carry_when_mem_eq_x() {
            let mut cpu = cpu!();
            cpu.x = 0xFF;
            cpu.cpx(Address::Immediate(0xFF));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cpx_sets_negative_with_bit7_result() {
            let mut cpu = cpu!();
            cpu.x = 0xFF;
            cpu.cpx(Address::Immediate(0x1F));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn cpy_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.y = 0x80;
            cpu.cpy(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn cpy_sets_carry_when_mem_lt_y() {
            let mut cpu = cpu!();
            cpu.y = 0xFF;
            cpu.cpy(Address::Immediate(0x01));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cpy_sets_carry_when_mem_eq_y() {
            let mut cpu = cpu!();
            cpu.y = 0xFF;
            cpu.cpy(Address::Immediate(0xFF));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn cpy_sets_negative_with_bit7_result() {
            let mut cpu = cpu!();
            cpu.y = 0xFF;
            cpu.cpy(Address::Immediate(0x1F));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn dec_decrements_memory() {
            let mut cpu = cpu!(0x00, vec![0x02]);
            cpu.dec(Address::Absolute(0x00));
            assert_eq!(cpu.mem.mem[0x00], 0x01);
        }

        #[test]
        fn dec_sets_zero_flag() {
            let mut cpu = cpu!(0x00, vec![0x01]);
            cpu.dec(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn dec_sets_negative_flag() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.dec(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn dec_wraps_underflows() {
            let mut cpu = cpu!(0x00, vec![0x00]);
            cpu.dec(Address::Absolute(0x00));
            assert_eq!(cpu.mem.mem[0], 0xFF);
        }

        #[test]
        fn dex_decrements_x() {
            let mut cpu = cpu!();
            cpu.x = 0x02;
            cpu.dex(Address::Implied);
            assert_eq!(cpu.x, 0x01);
        }

        #[test]
        fn dex_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.x = 0x01;
            cpu.dex(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn dex_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.x = 0xFF;
            cpu.dex(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn dex_wraps_underflows() {
            let mut cpu = cpu!();
            cpu.x = 0x00;
            cpu.dex(Address::Implied);
            assert_eq!(cpu.x, 0xFF);
        }

        #[test]
        fn dey_decrements_y() {
            let mut cpu = cpu!();
            cpu.y = 0x02;
            cpu.dey(Address::Implied);
            assert_eq!(cpu.y, 0x01);
        }

        #[test]
        fn dey_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.y = 0x01;
            cpu.dey(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn dey_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.y = 0xFF;
            cpu.dey(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn dey_wraps_underflows() {
            let mut cpu = cpu!();
            cpu.y = 0x00;
            cpu.dey(Address::Implied);
            assert_eq!(cpu.y, 0xFF);
        }

        #[test]
        fn eor_combines_mem_into_a() {
            let mut cpu = cpu!();
            cpu.a = 0b1010;
            cpu.eor(Address::Immediate(0b1100));
            assert_eq!(cpu.a, 0b0110);
        }

        #[test]
        fn eor_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x0F;
            cpu.eor(Address::Immediate(0x0F));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn eor_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x80;
            cpu.eor(Address::Immediate(0x08));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn inc_increments_memory() {
            let mut cpu = cpu!(0x00, vec![0x02]);
            cpu.inc(Address::Absolute(0x00));
            assert_eq!(cpu.mem.mem[0x00], 0x03);
        }

        #[test]
        fn inc_sets_zero_flag() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.inc(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn inc_sets_negative_flag() {
            let mut cpu = cpu!(0x00, vec![0x7F]);
            cpu.inc(Address::Absolute(0x00));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn inc_wraps_overflows() {
            let mut cpu = cpu!(0x00, vec![0xFF]);
            cpu.inc(Address::Absolute(0x00));
            assert_eq!(cpu.mem.mem[0], 0x00);
        }

        #[test]
        fn inx_increments_x() {
            let mut cpu = cpu!();
            cpu.x = 0x02;
            cpu.inx(Address::Implied);
            assert_eq!(cpu.x, 0x03);
        }

        #[test]
        fn inx_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.x = 0xFF;
            cpu.inx(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn inx_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.x = 0x7F;
            cpu.inx(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn inx_wraps_underflows() {
            let mut cpu = cpu!();
            cpu.x = 0xFF;
            cpu.inx(Address::Implied);
            assert_eq!(cpu.x, 0x00);
        }

        #[test]
        fn iny_increments_y() {
            let mut cpu = cpu!();
            cpu.y = 0x02;
            cpu.iny(Address::Implied);
            assert_eq!(cpu.y, 0x03);
        }

        #[test]
        fn iny_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.y = 0xFF;
            cpu.iny(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn iny_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.y = 0x7F;
            cpu.iny(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn iny_wraps_underflows() {
            let mut cpu = cpu!();
            cpu.y = 0xFF;
            cpu.iny(Address::Implied);
            assert_eq!(cpu.y, 0x00);
        }

        #[test]
        fn jmp_sets_pc() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x01, 0x02]);
            cpu.jmp(Address::Absolute(0x01));
            assert_eq!(cpu.pc, 0x0001);
        }

        #[test]
        fn jsr_sets_pc() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.s = 0xFF;

            cpu.jsr(Address::Absolute(0xABCD));
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn jsr_pushes_pc_to_stack() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.pc = 0xABCD;
            cpu.s = 0xFF;
            cpu.jsr(Address::Absolute(0x00));
            assert_eq!(cpu.mem.mem[0x01FF], 0xAB);
            assert_eq!(cpu.mem.mem[0x01FE], 0xCC);
        }

        #[test]
        fn lda_sets_a() {
            let mut cpu = cpu!();
            cpu.lda(Address::Immediate(0x42));
            assert_eq!(cpu.a, 0x42);
        }

        #[test]
        fn lda_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.lda(Address::Immediate(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn lda_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.lda(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn ldx_sets_x() {
            let mut cpu = cpu!();
            cpu.ldx(Address::Immediate(0x42));
            assert_eq!(cpu.x, 0x42);
        }

        #[test]
        fn ldx_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.ldx(Address::Immediate(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn ldx_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.ldx(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn ldy_sets_y() {
            let mut cpu = cpu!();
            cpu.ldy(Address::Immediate(0x42));
            assert_eq!(cpu.y, 0x42);
        }

        #[test]
        fn ldy_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.ldy(Address::Immediate(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn ldy_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.ldy(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn lsr_shifts_bits_right() {
            let mut cpu = cpu!();
            cpu.a = 0b10101010;
            cpu.lsr(Address::Accumulator);
            assert_eq!(cpu.a, 0b01010101);
        }

        #[test]
        fn lsr_sets_zero() {
            let mut cpu = cpu!();
            cpu.a = 0x01;
            cpu.lsr(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn lsr_sets_carry() {
            let mut cpu = cpu!();
            cpu.a = 0x01;
            cpu.lsr(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn ora_ors_bytes() {
            let mut cpu = cpu!();
            cpu.a = 0b11001100;
            cpu.ora(Address::Immediate(0b10101100));
            assert_eq!(cpu.a, 0b11101100);
        }

        #[test]
        fn ora_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x00;
            cpu.ora(Address::Immediate(0x00));
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn ora_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x00;
            cpu.ora(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn pha_pushes_accumulator_to_stack() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.a = 0x42;
            cpu.pha(Address::Implied);
            assert_eq!(cpu.mem.mem[0x01FF], 0x42);
        }

        #[test]
        fn pha_advances_stack_pointer() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.pha(Address::Implied);
            assert_eq!(cpu.s, 0xFE);
        }

        #[test]
        fn php_pushes_flags_to_stack() {
            let mut cpu = cpu!(0x42, vec![0; 0x0200]);
            cpu.php(Address::Implied);

            // Bits 3 & 4 are always set during PHP operation
            assert_eq!(cpu.mem.mem[0x01FF], 0x42 | 0x30);
        }

        #[test]
        fn php_advances_stack_pointer() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.php(Address::Implied);
            assert_eq!(cpu.s, 0xFE);
        }

        #[test]
        fn pla_reads_byte_from_stack() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0x42;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFE;
            cpu.pla(Address::Implied);
            assert_eq!(cpu.a, 0x42);
        }

        #[test]
        fn pla_advances_stack_pointer() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.s = 0xFE;
            cpu.pla(Address::Implied);
            assert_eq!(cpu.s, 0xFF);
        }

        #[test]
        fn pla_sets_zero_flag() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.s = 0xFE;
            cpu.pla(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn pla_sets_negative_flag() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0x80;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFE;
            cpu.pla(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn plp_sets_status_register() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0x42;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFE;
            cpu.plp(Address::Implied);

            // 4th bit is not modified by PLP
            // 5th bit is always high because it isn't real in the HW
            assert_eq!(cpu.flags, Flags::from_bits_retain(0x42 & 0xEF | 0x20));
        }

        #[test]
        fn plp_advances_stack_pointer() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0x42;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFE;
            cpu.plp(Address::Implied);
            assert_eq!(cpu.s, 0xFF);
        }

        #[test]
        fn rol_shifts_bits_left() {
            let mut cpu = cpu!();
            cpu.a = 0b01010101;
            cpu.rol(Address::Accumulator);
            assert_eq!(cpu.a, 0b10101010);
        }

        #[test]
        fn rol_fills_bit_0_with_carry_flag() {
            let mut cpu = cpu!();
            cpu.a = 0b11110000;
            cpu.flags.insert(Flags::CARRY);
            cpu.rol(Address::Accumulator);
            assert_eq!(cpu.a, 0b11100001);
        }

        #[test]
        fn rol_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x80;
            cpu.rol(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn rol_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x40;
            cpu.rol(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn rol_sets_carry_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x80;
            cpu.rol(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn ror_shifts_bits_right() {
            let mut cpu = cpu!();
            cpu.a = 0b01010101;
            cpu.ror(Address::Accumulator);
            assert_eq!(cpu.a, 0b00101010);
        }

        #[test]
        fn ror_fills_bit_7_with_carry_flag() {
            let mut cpu = cpu!();
            cpu.a = 0b11110000;
            cpu.flags.insert(Flags::CARRY);
            cpu.ror(Address::Accumulator);
            assert_eq!(cpu.a, 0b11111000);
        }

        #[test]
        fn ror_sets_negative_flag_with_carry() {
            let mut cpu = cpu!();
            cpu.flags.insert(Flags::CARRY);
            cpu.ror(Address::Accumulator);
            assert_eq!(cpu.a, 0x80);
        }

        #[test]
        fn ror_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x01;
            cpu.ror(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn ror_sets_carry_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x01;
            cpu.ror(Address::Accumulator);
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn rti_restores_flags() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0xCD;
            mem[0x01FE] = 0xAB;
            mem[0x01FD] = 0x42;

            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFC;
            cpu.rti(Address::Implied);

            // Add 0x20 to assertion since this bit is always high for emu
            assert_eq!(cpu.flags, Flags::from_bits_retain(0x42 | 0x20));
        }

        #[test]
        fn rti_restores_pc() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0xAB;
            mem[0x01FE] = 0xCD;
            mem[0x01FD] = 0x42;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFC;
            cpu.rti(Address::Implied);
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn rts_restores_pc() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0xAB;
            mem[0x01FE] = 0xCC;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFD;
            cpu.rts(Address::Implied);
            assert_eq!(cpu.pc, 0xABCD);
        }

        #[test]
        fn rts_advances_stack_pointer() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0xAB;
            mem[0x01FE] = 0xCC;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFD;
            cpu.rts(Address::Implied);
            assert_eq!(cpu.s, 0xFF);
        }

        #[test]
        fn sbc_subtracts_mem() {
            let mut cpu = cpu!(Flags::CARRY.bits(), vec![]);
            cpu.a = 0x7F;
            cpu.sbc(Address::Immediate(0x01));
            assert_eq!(cpu.a, 0x7E);
        }

        #[test]
        fn sbc_subtracts_borrow() {
            let mut cpu = cpu!(0x00, vec![]);
            cpu.a = 0x7F;
            cpu.sbc(Address::Immediate(0x01));
            assert_eq!(cpu.a, 0x7D);
        }

        #[test]
        fn sbc_wraps_byte_underflow() {
            let mut cpu = cpu!(Flags::CARRY.bits(), vec![]);
            cpu.a = 0x00;
            cpu.sbc(Address::Immediate(0x01));
            assert_eq!(cpu.a, 0xFF);
        }

        #[test]
        fn sbc_sets_v_for_overflow() {
            let mut cpu = cpu!(Flags::CARRY.bits(), vec![]);
            cpu.a = 0x00;
            cpu.sbc(Address::Immediate(0x80));
            assert!(cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn sbc_sets_v_for_underflow() {
            let mut cpu = cpu!(Flags::CARRY.bits(), vec![]);
            cpu.a = 0x80;
            cpu.sbc(Address::Immediate(0x01));
            assert!(cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn sbc_clears_v_for_non_overflow() {
            let flags = Flags::CARRY.bits() | Flags::OVERFLOW.bits();
            let mut cpu = cpu!(flags, vec![]);
            cpu.a = 0x0A;
            cpu.sbc(Address::Immediate(0x05));
            assert!(!cpu.flags.intersects(Flags::OVERFLOW));
        }

        #[test]
        fn sbc_clears_c_for_borrow() {
            let mut cpu = cpu!(Flags::CARRY.bits(), vec![]);
            cpu.a = 0x02;
            cpu.sbc(Address::Immediate(0x05));
            assert!(!cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn sbc_sets_c_for_no_borrow() {
            let mut cpu = cpu!(Flags::CARRY.bits(), vec![]);
            cpu.a = 0x0A;
            cpu.sbc(Address::Immediate(0x05));
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn sec_sets_carry_flag() {
            let mut cpu = cpu!();
            cpu.sec(Address::Implied);
            assert!(cpu.flags.intersects(Flags::CARRY));
        }

        #[test]
        fn sed_sets_decimal_flag() {
            let mut cpu = cpu!();
            cpu.sed(Address::Implied);
            assert!(cpu.flags.intersects(Flags::DECIMAL));
        }

        #[test]
        fn sei_sets_interrupt_flag() {
            let mut cpu = cpu!();
            cpu.sei(Address::Implied);
            assert!(cpu.flags.intersects(Flags::IRQ_DISABLE));
        }

        #[test]
        fn sta_stores_acc_to_memory() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x01]);
            cpu.a = 0x42;
            cpu.sta(Address::Absolute(0x01));
            assert_eq!(cpu.mem.mem[0x01], 0x42);
        }

        #[test]
        fn stx_stores_x_to_memory() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x01]);
            cpu.x = 0x42;
            cpu.stx(Address::Absolute(0x01));
            assert_eq!(cpu.mem.mem[0x01], 0x42);
        }

        #[test]
        fn sty_stores_y_to_memory() {
            let mut cpu = cpu!(0x00, vec![0x00, 0x01]);
            cpu.y = 0x42;
            cpu.sty(Address::Absolute(0x01));
            assert_eq!(cpu.mem.mem[0x01], 0x42);
        }

        #[test]
        fn tax_stores_acc_in_x() {
            let mut cpu = cpu!();
            cpu.a = 0x42;
            cpu.tax(Address::Implied);
            assert_eq!(cpu.x, 0x42);
        }

        #[test]
        fn tax_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x00;
            cpu.tax(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn tax_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x80;
            cpu.tax(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn tay_stores_acc_in_y() {
            let mut cpu = cpu!();
            cpu.a = 0x42;
            cpu.tay(Address::Implied);
            assert_eq!(cpu.y, 0x42);
        }

        #[test]
        fn tay_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x00;
            cpu.tay(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn tay_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.a = 0x80;
            cpu.tay(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn tsx_sets_x_to_stack_pointer() {
            let mut cpu = cpu!(0x00);
            cpu.s = 0xFE;
            cpu.tsx(Address::Implied);
            assert_eq!(cpu.x, 0xFE);
        }

        #[test]
        fn tsx_doesnt_update_stack_pointer() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.s = 0xFE;
            cpu.tsx(Address::Implied);
            assert_eq!(cpu.s, 0xFE);
        }

        #[test]
        fn tsx_sets_zero_flag() {
            let mut cpu = cpu!(0x00, vec![0; 0x0200]);
            cpu.s = 0x00;
            cpu.tsx(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn tsx_sets_negative_flag() {
            let mut mem = vec![0; 0x0200];
            mem[0x01FF] = 0x80;
            let mut cpu = cpu!(0x00, mem);
            cpu.s = 0xFE;
            cpu.tsx(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn txa_sets_x_to_a() {
            let mut cpu = cpu!();
            cpu.x = 0x42;
            cpu.txa(Address::Implied);
            assert_eq!(cpu.a, 0x42);
        }

        #[test]
        fn txa_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.txa(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn txa_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.x = 0x80;
            cpu.txa(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }

        #[test]
        fn txs_copies_x_into_stack_pointer() {
            let mut cpu = cpu!(0x00);
            cpu.x = 0x42;
            cpu.txs(Address::Implied);
            assert_eq!(cpu.s, 0x42);
        }

        #[test]
        fn tya_sets_y_to_a() {
            let mut cpu = cpu!();
            cpu.y = 0x42;
            cpu.tya(Address::Implied);
            assert_eq!(cpu.a, 0x42);
        }

        #[test]
        fn tya_sets_zero_flag() {
            let mut cpu = cpu!();
            cpu.tya(Address::Implied);
            assert!(cpu.flags.intersects(Flags::ZERO));
        }

        #[test]
        fn tya_sets_negative_flag() {
            let mut cpu = cpu!();
            cpu.y = 0x80;
            cpu.tya(Address::Implied);
            assert!(cpu.flags.intersects(Flags::NEGATIVE));
        }
    }
}
