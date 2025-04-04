pub use crate::{address::Address, operation::Op};
use std::cmp::Ordering;

pub const PRG_ROM_BASE: usize = 0x4020;

#[derive(Debug, Copy, Clone)]
pub enum AddressOrOp {
    Address(Address),
    Op(Op),
    Unknown(u8),
}

pub struct Disassembler {
    ops: Vec<(usize, AddressOrOp)>,
}

impl Disassembler {
    pub fn new(src: &[u8]) -> Self {
        debug_assert!(
            src.len() == 0xBFE0,
            "ROM range should span from 0x4020 to 0xFFFF"
        );

        let ops = parse_ops(src)
            .into_iter()
            .enumerate()
            .filter_map(|(addr, op)| op.map(|op| (addr + PRG_ROM_BASE, op)))
            .collect();

        Self { ops }
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Returns the index of the given address in ROM, or the address of the prior op if no
    /// operation falls on the given address.
    ///
    /// Time complexity: O(lg n), where n is the byte size of the ROM
    pub fn index_of(&self, addr: u16) -> usize {
        let addr = addr as usize;
        let mut lo = 0;
        let mut hi = self.ops.len() - 1;

        while lo < hi {
            let mid = lo + ((hi - lo) / 2);
            let mid_val = self.ops[mid].0;

            match addr.cmp(&mid_val) {
                Ordering::Less => hi = mid - 1,
                Ordering::Equal => return mid,
                Ordering::Greater => lo += mid + 1,
            }
        }

        lo
    }

    pub fn get(&self, index: usize) -> Option<(u16, AddressOrOp)> {
        self.ops.get(index).map(|op| (op.0 as u16, op.1))
    }
}

fn parse_ops(src: &[u8]) -> Vec<Option<AddressOrOp>> {
    let mut roots = Vec::with_capacity(3);
    let mut ret = src
        .iter()
        .copied()
        .map(|byte| Some(AddressOrOp::Unknown(byte)))
        .collect::<Vec<_>>();

    // Populate NMI, Reset, and IRQ/BRK vectors
    for vec in [0xFFFA, 0xFFFC, 0xFFFE] {
        debug_assert!(vec >= PRG_ROM_BASE);
        let (root, addr) = populate_vector(vec, src);
        roots.push(root);
        ret[vec - PRG_ROM_BASE] = Some(addr);
        ret[vec + 1 - PRG_ROM_BASE] = None;
    }

    while let Some(root) = roots.pop() {
        if matches!(
            ret[root - PRG_ROM_BASE],
            Some(AddressOrOp::Address(_)) | Some(AddressOrOp::Op(_))
        ) {
            continue;
        }

        let op = src_addr(src, root);
        let lo = src_addr(src, root + 1);
        let hi = src_addr(src, root + 2);

        let op = Op::from_bytes(op, lo, hi);
        ret[root - PRG_ROM_BASE] = Some(AddressOrOp::Op(op));

        let byte_len = op.byte_len();
        for i in 1..byte_len {
            ret[root + i - PRG_ROM_BASE] = None;
        }

        let (next1, next2) = op.next_addresses(root);

        if let Some(next) = next1 {
            roots.push(next);
        }

        if let Some(next) = next2 {
            roots.push(next);
        }
    }

    ret
}

/// Calculate the address which a vector would evaluate, and return a tuple of that address as well
/// as the same location in absolute addressing format
fn populate_vector(base: usize, src: &[u8]) -> (usize, AddressOrOp) {
    let lo = src_addr(src, base) as usize;
    let hi = src_addr(src, base + 1) as usize;
    let word = (hi << 8) | lo;

    let addr = Address::absolute_from_le(lo as u8, hi as u8);
    let addr = AddressOrOp::Address(addr);
    (word, addr)
}

fn src_addr(src: &[u8], addr: usize) -> u8 {
    debug_assert!(addr >= PRG_ROM_BASE);
    let addr = addr - PRG_ROM_BASE;
    src[addr]
}
