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

/// Accesses a 0-indexed bit of an expression, yielding true if the given bit is
/// set or false otherwise
macro_rules! bit {
    ($val:expr, $bit:expr) => { $val & (1 << $bit) != 0 };
}

/// Returns the value of the given bit, 1 or 0
macro_rules! bitn {
    ($val:expr, $bit:expr) => { ($val & (1 << $bit)) >> $bit };
}

/// Set the given bit (0 indexed) of the byte to 1
macro_rules! set_bit {
    ($val:expr, $bit:literal) => { $val |= (1 << $bit) };
}

/// Set the given bit (0 indexed) of the byte to 0
macro_rules! clear_bit {
    ($val:expr, $bit:literal) => { $val &= !(1 << $bit) };
}

macro_rules! toggle_bit {
    ($val:expr, $bit:literal, $hi:expr) => {
        if $hi {
            set_bit!($val, $bit)
        } else {
            clear_bit!($val, $bit)
        }
    };
}

macro_rules! mask {
    ($val:expr, $mask:literal) => { $val & $mask };
}

#[cfg(test)]
mod tests {
    #[test]
    fn word_combines_bytes() {
        assert_eq!(word!(0xFF, 0xFF), 0xFFFF);
    }

    #[test]
    fn word_accepts_lo_first() {
        assert_eq!(word!(0x01, 0x02), 0x0201);
    }
}
