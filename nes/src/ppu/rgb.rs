static COLORS: [Rgb; 0x40] = [
    Rgb {
        r: 84,
        g: 84,
        b: 84,
    },
    Rgb {
        r: 0,
        g: 30,
        b: 116,
    },
    Rgb {
        r: 8,
        g: 16,
        b: 144,
    },
    Rgb {
        r: 48,
        g: 0,
        b: 136,
    },
    Rgb {
        r: 68,
        g: 0,
        b: 100,
    },
    Rgb { r: 92, g: 0, b: 48 },
    Rgb { r: 84, g: 4, b: 0 },
    Rgb { r: 60, g: 24, b: 0 },
    Rgb { r: 32, g: 42, b: 0 },
    Rgb { r: 8, g: 58, b: 0 },
    Rgb { r: 0, g: 64, b: 0 },
    Rgb { r: 0, g: 60, b: 0 },
    Rgb { r: 0, g: 50, b: 60 },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb {
        r: 152,
        g: 150,
        b: 152,
    },
    Rgb {
        r: 8,
        g: 76,
        b: 196,
    },
    Rgb {
        r: 48,
        g: 50,
        b: 236,
    },
    Rgb {
        r: 92,
        g: 30,
        b: 228,
    },
    Rgb {
        r: 136,
        g: 20,
        b: 176,
    },
    Rgb {
        r: 160,
        g: 20,
        b: 100,
    },
    Rgb {
        r: 152,
        g: 34,
        b: 32,
    },
    Rgb {
        r: 120,
        g: 60,
        b: 0,
    },
    Rgb { r: 84, g: 90, b: 0 },
    Rgb {
        r: 40,
        g: 114,
        b: 0,
    },
    Rgb { r: 8, g: 124, b: 0 },
    Rgb {
        r: 0,
        g: 118,
        b: 40,
    },
    Rgb {
        r: 0,
        g: 102,
        b: 120,
    },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb {
        r: 236,
        g: 238,
        b: 236,
    },
    Rgb {
        r: 76,
        g: 154,
        b: 236,
    },
    Rgb {
        r: 120,
        g: 124,
        b: 236,
    },
    Rgb {
        r: 176,
        g: 98,
        b: 236,
    },
    Rgb {
        r: 228,
        g: 84,
        b: 236,
    },
    Rgb {
        r: 236,
        g: 88,
        b: 180,
    },
    Rgb {
        r: 236,
        g: 106,
        b: 100,
    },
    Rgb {
        r: 212,
        g: 136,
        b: 32,
    },
    Rgb {
        r: 160,
        g: 170,
        b: 0,
    },
    Rgb {
        r: 116,
        g: 196,
        b: 0,
    },
    Rgb {
        r: 76,
        g: 208,
        b: 32,
    },
    Rgb {
        r: 56,
        g: 204,
        b: 108,
    },
    Rgb {
        r: 56,
        g: 180,
        b: 204,
    },
    Rgb {
        r: 60,
        g: 60,
        b: 60,
    },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
    Rgb {
        r: 168,
        g: 204,
        b: 236,
    },
    Rgb {
        r: 188,
        g: 188,
        b: 236,
    },
    Rgb {
        r: 212,
        g: 178,
        b: 236,
    },
    Rgb {
        r: 236,
        g: 174,
        b: 236,
    },
    Rgb {
        r: 236,
        g: 174,
        b: 212,
    },
    Rgb {
        r: 236,
        g: 180,
        b: 176,
    },
    Rgb {
        r: 228,
        g: 196,
        b: 144,
    },
    Rgb {
        r: 204,
        g: 210,
        b: 120,
    },
    Rgb {
        r: 180,
        g: 222,
        b: 120,
    },
    Rgb {
        r: 168,
        g: 226,
        b: 144,
    },
    Rgb {
        r: 152,
        g: 226,
        b: 180,
    },
    Rgb {
        r: 160,
        g: 214,
        b: 228,
    },
    Rgb {
        r: 160,
        g: 162,
        b: 160,
    },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb { r: 0, g: 0, b: 0 },
];

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn from_byte(byte: u8) -> &'static Rgb {
        if byte >= 0x40 {
            panic!("Attempted to load invalid color {:02X}", byte);
        }
        &COLORS[byte as usize]
    }
}
