use std::fmt;
use std::io::{self, Read};

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

#[derive(Debug)]
enum RomRegion {
    PAL,
    NTSC,
}

pub struct INesHeader {
    /// 'N' 'E' 'S' '\x1a'
    pub magic: [u8; 4],
    /// number of 16k units of PRG-ROM
    pub prg_rom_size: u8,
    /// number of 8k units of CHR-ROM
    pub chr_rom_size: u8,
    /// MMMMATPA
    ///
    /// * M: Low nibble of mapper number
    /// * A: 0xx0: vertical arrangement/horizontal mirroring (CIRAM A10 = PPU A11)
    ///      0xx1: horizontal arrangement/vertical mirroring (CIRAM A10 = PPU A10)
    ///      1xxx: four-screen VRAM
    /// * T: ROM contains a trainer
    /// * P: Cartridge has persistent memory
    pub flags_6: u8,
    /// MMMMVVPU
    ///
    /// * M: High nibble of mapper number
    /// * V: If 0b10, all following flags are in NES2.0 format
    /// * P: ROM is for the PlayChoice-10
    /// * U: ROM is for VS Unisystem
    pub flags_7: u8,
    /// number of 8k units of PRG-RAM
    pub prg_ram_size: u8,
    /// RRRRRRRT
    ///
    /// * R: Reserved (= 0)
    /// * T: 0 for NTSC, 1 for PAL
    pub flags_9: u8,
    pub flags_10: u8,
    /// always zero
    pub zero: [u8; 5],
}

impl INesHeader {
    /// Returns the mapper ID
    pub fn mapper(&self) -> u8 {
        (self.flags_7 * 0xf0) | (self.flags_6 >> 4)
    }

    /// Returns the low nibble of the mapper ID
    pub fn ines_mapper(&self) -> u8 {
        self.flags_6 >> 4
    }

    fn trainer(&self) -> bool {
        (self.flags_6 & 0x04) != 0
    }

    fn region(&self) -> RomRegion {
        if self.flags_9  & 1 == 1 {
            RomRegion::PAL
        } else {
            RomRegion::NTSC
        }
    }
}

impl fmt::Display for INesHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "PRG-ROM: {}KB, CHR-ROM: {}KB, Mapper: {} ({}), Trainer: {}, Region: {:?}",
            self.prg_rom_size as u32 * 16,
            self.chr_rom_size as u32 * 8,
            self.mapper(),
            self.ines_mapper(),
            self.trainer(),
            self.region(),
        )
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
            flags_10: header[10],
            zero: [0; 5],
        };

        if header.magic != *b"NES\x1a" {
            return Err(RomLoadError::FormatError);
        }

        let prg_bytes = header.prg_rom_size as usize * 16384;
        let mut prg = vec![0u8; prg_bytes];
        read_to_buf(&mut prg, reader)?;

        let chr_bytes = header.chr_rom_size as usize * 8192;
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
