use std::{
    fmt::Display,
    io::{self, Read},
};

/// The size of one bank of PRG ROM: 16KiB (16384b)
pub const PRG_ROM_BANK_SIZE: usize = 0x4000;
/// The size of one bank of PRG RAM: 8KiB (8192b)
pub const _PRG_RAM_BANK_SIZE: usize = 0x2000;
/// The size of one bank of PRG ROM: 8KiB (8192b)
pub const CHR_ROM_BANK_SIZE: usize = 0x2000;

#[derive(Debug)]
pub enum RomLoadError {
    /// IO Error while reading the ROM image
    IoError(io::Error),
    /// The ROM image has an invalid format
    FormatError,
}

impl From<io::Error> for RomLoadError {
    fn from(err: io::Error) -> Self {
        RomLoadError::IoError(err)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RomRegion {
    Pal,
    Ntsc,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum NametableMirror {
    Horizontal,
    Vertical,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum INesVersion {
    INes1,
    INes2,
}

impl Display for INesVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = if *self == Self::INes1 { "1.0" } else { "2.0" };
        write!(f, "{n}")
    }
}

#[derive(Debug, Clone)]
pub struct INesHeader {
    /// 'N' 'E' 'S' '\x1a'
    magic: [u8; 4],
    /// number of 16k units of PRG-ROM
    prg_rom_size: u8,
    /// number of 8k units of CHR-ROM
    chr_rom_size: u8,
    /// MMMMATPA
    ///
    /// * M: Low nibble of mapper number
    /// * A: 0xx0: vertical arrangement/horizontal mirroring (CIRAM A10 = PPU A11)
    ///      0xx1: horizontal arrangement/vertical mirroring (CIRAM A10 = PPU A10)
    ///      1xxx: four-screen VRAM
    /// * T: ROM contains a trainer
    /// * P: Cartridge has persistent memory
    flags_6: u8,
    /// MMMMVVPU
    ///
    /// * M: High nibble of mapper number
    /// * V: If 0b10, all following flags are in NES2.0 format
    /// * P: ROM is for the PlayChoice-10
    /// * U: ROM is for VS Unisystem
    flags_7: u8,
    /// number of 8k units of PRG-RAM
    prg_ram_size: u8,
    /// RRRRRRRT
    ///
    /// * R: Reserved (= 0)
    /// * T: 0 for NTSC, 1 for PAL
    flags_9: u8,
    _flags_10: u8,
    /// always zero
    _zero: [u8; 5],
}

impl INesHeader {
    /// Returns the mapper ID
    pub fn mapper(&self) -> u8 {
        (self.flags_7 * 0xf0) | (self.flags_6 >> 4)
    }

    pub fn ines_version(&self) -> INesVersion {
        if (self.flags_7 & 0b1100) >> 2 == 2 {
            INesVersion::INes2
        } else {
            INesVersion::INes1
        }
    }

    pub fn has_trainer(&self) -> bool {
        (self.flags_6 & 0x04) != 0
    }

    pub fn mirroring(&self) -> NametableMirror {
        if self.flags_6 & 1 == 0 {
            NametableMirror::Horizontal
        } else {
            NametableMirror::Vertical
        }
    }

    pub fn chr_banks(&self) -> usize {
        self.chr_rom_size.into()
    }

    pub fn prg_banks(&self) -> usize {
        self.prg_rom_size.into()
    }

    pub fn prg_ram_size(&self) -> usize {
        self.prg_ram_size.into()
    }

    pub fn has_battery_save(&self) -> bool {
        self.flags_6 & 0b10 != 0
    }

    pub fn region(&self) -> RomRegion {
        if self.flags_9 & 1 == 1 {
            RomRegion::Pal
        } else {
            RomRegion::Ntsc
        }
    }
}

/// A ROM image
pub struct Rom {
    pub header: INesHeader,
    /// PRG ROM
    pub prg: Vec<u8>,
    /// PRG RAM
    pub chr: Vec<u8>,
}

impl Rom {
    pub fn from_path(reader: &mut dyn Read) -> Result<Rom, RomLoadError> {
        let mut header = [0u8; 16];
        read_to_buf(&mut header, reader)?;

        let header = INesHeader {
            magic: [header[0], header[1], header[2], header[3]],
            prg_rom_size: header[4],
            chr_rom_size: header[5],
            flags_6: header[6],
            flags_7: header[7],
            prg_ram_size: header[8],
            flags_9: header[9],
            _flags_10: header[10],
            _zero: [0; 5],
        };

        if header.magic != *b"NES\x1a" {
            Err(RomLoadError::FormatError)?;
        }

        let prg_bytes = header.prg_rom_size as usize * PRG_ROM_BANK_SIZE;
        let mut prg = vec![0u8; prg_bytes];
        read_to_buf(&mut prg, reader)?;

        let chr_bytes = header.chr_rom_size as usize * CHR_ROM_BANK_SIZE;
        let mut chr = vec![0u8; chr_bytes];
        read_to_buf(&mut chr, reader)?;

        Ok(Rom { header, prg, chr })
    }
}

fn read_to_buf(buf: &mut [u8], reader: &mut dyn Read) -> io::Result<()> {
    let mut total = 0;
    while total < buf.len() {
        let count = reader.read(&mut buf[total..])?;
        if count == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "unexpected eof"));
        }
        total += count;
    }
    Ok(())
}
