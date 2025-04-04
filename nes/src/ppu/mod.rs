mod oam;
mod registers;
mod rgb;
mod shifters;
mod sprite;
mod vram;

use crate::frame_buffer::Frame;
use crate::mapper::ChrMem;
use crate::mem::Mem;
use log::warn;
use oam::Oam;
use registers::Vtwx;
pub use registers::{PpuControl, PpuMask, PpuStatus};
pub use rgb::Rgb;
use shifters::PatternShifter;
use sprite::{SpritePriority, SpriteShift};
use vram::Vram;

#[derive(Default)]
pub struct PpuResult {
    pub new_frame: bool,
    pub scanline_irq: bool,
    pub vblank_nmi: bool,
}

#[derive(Default)]
struct PpuPosition {
    cycle: u64,
    frame: u64,
    pixel: u16,
    scanline: u16,
}

impl PpuPosition {
    fn is_warmed_up(&self) -> bool {
        self.frame > 0
    }

    fn is_in_vblank(&self) -> bool {
        self.scanline >= 240 && self.scanline < 261 || self.scanline == 261 && self.pixel == 0
    }

    fn step(&mut self, rendering_enabled: bool) {
        let is_last_scanline = self.scanline == 261;
        let is_odd_frame = self.frame % 2 == 1;
        let is_last_pixel = self.pixel == 340
            || self.pixel == 339 && rendering_enabled && is_last_scanline && is_odd_frame;

        self.cycle += 1;

        if is_last_pixel {
            self.pixel = 0;

            if is_last_scanline {
                self.scanline = 0;
                self.frame += 1;
            } else {
                self.scanline += 1;
            }
        } else {
            self.pixel += 1;
        }
    }
}

macro_rules! is_any {
    ($test:expr, [ $( $val:literal ),* $(,)? ] ) => {
        match $test {
            $( $val => true, )*
            _ => false,
        }
    }
}

fn nt_cycle(px: u16) -> bool {
    is_any!(
        px,
        [
            1, 9, 17, 25, 33, 41, 49, 57, 65, 73, 81, 89, 97, 105, 113, 121, 129, 137, 145, 153,
            161, 169, 177, 185, 193, 201, 209, 217, 225, 233, 241, 321, 329,
        ]
    )
}

fn at_cycle(px: u16) -> bool {
    is_any!(
        px,
        [
            3, 11, 19, 27, 35, 43, 51, 59, 67, 75, 83, 91, 99, 107, 115, 123, 131, 139, 147, 155,
            163, 171, 179, 187, 195, 203, 211, 219, 227, 235, 243, 323, 331,
        ]
    )
}

fn pat_lo_cycle(px: u16) -> bool {
    is_any!(
        px,
        [
            5, 13, 21, 29, 37, 45, 53, 61, 69, 77, 85, 93, 101, 109, 117, 125, 133, 141, 149, 157,
            165, 173, 181, 189, 197, 205, 213, 221, 229, 237, 245, 325, 333,
        ]
    )
}

fn pat_hi_cycle(px: u16) -> bool {
    is_any!(
        px,
        [
            7, 15, 23, 31, 39, 47, 55, 63, 71, 79, 87, 95, 103, 111, 119, 127, 135, 143, 151, 159,
            167, 175, 183, 191, 199, 207, 215, 223, 231, 239, 247, 327, 335,
        ]
    )
}

fn garbage_nt_cycle(px: u16) -> bool {
    is_any!(px, [337, 339])
}

fn inc_scrollh_cycle(px: u16) -> bool {
    is_any!(
        px,
        [
            8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160,
            168, 176, 184, 192, 200, 208, 216, 224, 232, 240, 248, 328, 336,
        ]
    )
}

fn pattern_latch_cycle(px: u16) -> bool {
    is_any!(
        px,
        [
            9, 17, 25, 33, 41, 49, 57, 65, 73, 81, 89, 97, 105, 113, 121, 129, 137, 145, 153, 161,
            169, 177, 185, 193, 201, 209, 217, 225, 233, 241, 249, 257, 329, 337,
        ]
    )
}

fn sprite_nt_cycle(px: u16) -> bool {
    is_any!(px, [257, 265, 273, 281, 289, 297, 305, 313])
}

fn sprite_at_cycle(px: u16) -> bool {
    is_any!(px, [259, 267, 275, 283, 291, 299, 307, 315])
}

fn sprite_lo_cycle(px: u16) -> bool {
    is_any!(px, [261, 269, 277, 285, 293, 301, 309, 317])
}

fn sprite_hi_cycle(px: u16) -> bool {
    is_any!(px, [263, 271, 279, 287, 295, 303, 311, 319])
}

fn sprite_attr_cycle(px: u16) -> bool {
    is_any!(px, [259, 267, 275, 283, 291, 299, 307, 315])
}

fn sprite_x_cycle(px: u16) -> bool {
    is_any!(px, [260, 268, 276, 284, 292, 300, 308, 316])
}

pub struct Ppu {
    pub ppuctrl: PpuControl,
    pub ppumask: PpuMask,
    pub ppustatus: PpuStatus,
    pub vram: Vram,
    pub oam: Oam,
    pub screen: Frame,

    pos: PpuPosition,
    ppu_data_buffer: u8,
    vtwx: Vtwx,
    shifter: PatternShifter,
    sprite_shifters: [SpriteShift; 8],
    immediate_nmi: bool,
}

impl Ppu {
    pub fn new(mapper: ChrMem, frame_buffer: Frame) -> Self {
        Self {
            pos: PpuPosition::default(),

            ppuctrl: PpuControl::new(),
            ppumask: PpuMask::default(),
            ppustatus: PpuStatus::default(),
            ppu_data_buffer: 0,

            vram: Vram::new(mapper),
            oam: Oam::new(),
            vtwx: Vtwx::new(),
            shifter: PatternShifter::default(),
            sprite_shifters: [SpriteShift::default(); 8],
            immediate_nmi: false,

            screen: frame_buffer,
        }
    }

    /// Get the scanline that will be rendered next, 0-241
    pub fn scanline(&self) -> u16 {
        self.pos.scanline
    }

    /// Get the pixel that will be rendered next, 0-340
    pub fn pixel(&self) -> u16 {
        self.pos.pixel
    }

    pub fn cycle(&self) -> u64 {
        self.pos.cycle
    }

    pub fn frame(&self) -> u64 {
        self.pos.frame
    }

    pub fn scroll_x(&self) -> u8 {
        self.vtwx.fine_x()
    }

    pub fn scroll_y(&self) -> u8 {
        self.vtwx.fine_y() as u8
    }

    pub fn step(&mut self) -> PpuResult {
        let mut result = PpuResult::default();

        if self.immediate_nmi {
            self.immediate_nmi = false;
            result.vblank_nmi = true;
        }

        let rendering_enabled = self.ppumask.rendering_enabled();
        let line = self.scanline();
        let px = self.pixel();
        let is_render_line = line < 240;
        let is_prerender_line = line == 261;
        let is_idle_line = line >= 240 && !is_prerender_line;

        if pattern_latch_cycle(px) {
            self.shifter.load_pattern_latches();
        }

        if is_render_line && px < 256 {
            self.output_pixel(line, px);
        }

        // Flags
        if line == 241 && px == 1 {
            if self.ppuctrl.generate_nmi() {
                result.vblank_nmi = true;
            }

            self.ppustatus.set_vblank_started(true);
        } else if is_prerender_line && px == 1 {
            result.new_frame = true;
            self.ppustatus.set_sprite_overflow(false);
            self.ppustatus.set_sprite_zero_hit(false);
            self.ppustatus.set_vblank_started(false);
        }

        if rendering_enabled && !is_idle_line {
            // vtwx updates
            if px == 256 {
                self.vtwx.increment_v();
            } else if inc_scrollh_cycle(px) {
                self.vtwx.increment_h();
            } else if is_prerender_line && (280..=304).contains(&px) {
                self.vtwx.copy_v();
            } else if px == 257 {
                self.vtwx.copy_h();
            }

            // background fetches
            if nt_cycle(px) {
                let nt_byte = self.fetch_nametable();
                self.shifter.store_nametable(nt_byte);
            } else if garbage_nt_cycle(px) {
                self.fetch_nametable();
            } else if at_cycle(px) {
                let at_byte = self.fetch_attribute_table();
                self.shifter.store_attribute(at_byte);
            } else if pat_lo_cycle(px) {
                let lo = self.fetch_bg_lo();
                self.shifter.load_pattern_lo(lo);
            } else if pat_hi_cycle(px) {
                let hi = self.fetch_bg_hi();
                self.shifter.load_pattern_hi(hi);
            }

            // sprite fetches
            if px > 256 && px <= 320 {
                self.oam.reset_addr();
            }

            if sprite_attr_cycle(px) {
                let sprite_idx = (px / 8 - 32) as usize;
                let attr = self.oam.sprite_attr(sprite_idx);
                self.sprite_shifters[sprite_idx].set_attributes(attr);
            } else if sprite_x_cycle(px) {
                let sprite_idx = (px / 8 - 32) as usize;
                let x = self.oam.sprite_x(sprite_idx);
                self.sprite_shifters[sprite_idx].set_x(x);
            }

            if is_render_line && px == 1 {
                self.oam.reset_oam2();
            } else if is_render_line && px > 64 && px <= 256 {
                let overflow = self.oam.sprite_eval(line);
                if overflow {
                    self.ppustatus.set_sprite_overflow(true);
                }
            } else if sprite_nt_cycle(px) {
                self.fetch_nametable();
            } else if sprite_at_cycle(px) {
                self.fetch_attribute_table();
            } else if sprite_lo_cycle(px) {
                let lo = self.fetch_sprite_lo(line, px);
                let sprite_idx = (px / 8 - 32) as usize;
                let is_dummy_read = self.oam.sprite_y(sprite_idx) as u16 > 0xEE;
                let lo = if is_dummy_read { 0x00 } else { lo };
                self.sprite_shifters[sprite_idx].set_pattern_lo(lo);
            } else if sprite_hi_cycle(px) {
                let hi = self.fetch_sprite_hi(line, px);
                let sprite_idx = (px / 8 - 32) as usize;
                let is_dummy_read = self.oam.sprite_y(sprite_idx) as u16 > 0xEE;
                let hi = if is_dummy_read { 0x00 } else { hi };
                self.sprite_shifters[sprite_idx].set_pattern_hi(hi);
            }
        }

        if (2..=257).contains(&px) || (332..=337).contains(&px) {
            self.shifter.shift();
        }

        self.pos.step(rendering_enabled);

        result
    }

    /// For debugging purposes only!
    /// Set the PPU's internal cycle count, current pixel, and frame from a CPU cycle count
    ///
    /// This cannot be done accurately, because the number of cycles in one PPU frame depends
    /// on whether rendering was enabled during that time (and if so, for how many cycles).
    /// Thus, this function is mostly only helpful for initializing the PPU during debugging
    /// sessions. The nes emulation code never calls this directly.
    pub fn set_cycle_from_cpu(&mut self, cpu_cycle: u64) {
        // This isn't exact; it's the number of cycles in a frame assuming rendering is off.
        const CYCLES_PER_FRAME: u64 = 89342;

        let ppu_cycle = cpu_cycle * 3;
        let ppu_frame = ppu_cycle / CYCLES_PER_FRAME;
        let ppu_pixel = ppu_cycle % CYCLES_PER_FRAME;
        self.pos.frame = ppu_frame;
        self.pos.pixel = ppu_pixel as u16;
        self.pos.cycle = ppu_cycle;
    }

    fn fetch_nametable(&mut self) -> u8 {
        let tile_addr = self.vtwx.tile_addr();
        self.vram.loadb(tile_addr)
    }

    fn fetch_attribute_table(&mut self) -> u8 {
        let attr_addr = self.vtwx.attr_addr();
        let quadrant = self.vtwx.attr_quadrant();
        let attr_byte = self.vram.loadb(attr_addr);

        match quadrant {
            0x00 => attr_byte & 0x03,        // top left
            0x01 => (attr_byte & 0x0C) >> 2, // top right
            0x02 => (attr_byte & 0x30) >> 4, // bottom left
            0x03 => (attr_byte & 0xC0) >> 6, // bottom right
            _ => unreachable!(),
        }
    }

    fn fetch_bg_lo(&mut self) -> u8 {
        let addr = self.pattern_addr(0);
        self.vram.loadb(addr)
    }

    fn fetch_bg_hi(&mut self) -> u8 {
        let addr = self.pattern_addr(8);
        self.vram.loadb(addr)
    }

    fn fetch_sprite_lo(&mut self, line: u16, px: u16) -> u8 {
        let sprite_idx = px / 8 - 32;
        let addr = self.sprite_pattern_addr(sprite_idx as usize, line, 0);
        self.vram.loadb(addr)
    }

    fn fetch_sprite_hi(&mut self, line: u16, px: u16) -> u8 {
        let sprite_idx = px / 8 - 32;
        let addr = self.sprite_pattern_addr(sprite_idx as usize, line, 8);
        self.vram.loadb(addr)
    }

    fn pattern_addr(&self, bit_plane: u16) -> u16 {
        let fine_y = self.vtwx.fine_y();
        let nt_byte = self.shifter.nametable();
        let pattern_table_index = (nt_byte as u16) << 4;
        let pattern_table = self.ppuctrl.background_pattern_table_address();

        pattern_table | pattern_table_index | bit_plane | fine_y
    }

    fn sprite_pattern_addr(&self, sprite_idx: usize, line: u16, bit_plane: u16) -> u16 {
        let line = if line == 261 { 0 } else { line };
        let sprite_y = self.oam.sprite_y(sprite_idx) as u16;
        let pattern_table_index = (self.oam.sprite_addr(sprite_idx) as u16) << 4;
        let mirror_y = bit!(self.oam.sprite_attr(sprite_idx), 7);

        let fine_y = if sprite_y > line {
            0
        } else if mirror_y {
            // TODO: should this be saturating? Does it matter?
            7u16.saturating_sub(line - sprite_y)
        } else {
            line - sprite_y
        };

        let pattern_table = self.ppuctrl.sprite_pattern_table_address();
        pattern_table | pattern_table_index | bit_plane | fine_y
    }

    fn sprite_pixel(&mut self, px: u16) -> Option<(&'static Rgb, SpritePriority, usize)> {
        let mut ret = None;
        let sprites_enabled = self.ppumask.sprite_rendering_enabled();
        let leftmost_shown = self.ppumask.hide_first_sprite_tile();

        // Sprites disabled; no sprite 0 hit & no output
        if !sprites_enabled || px < 8 && !leftmost_shown {
            for shifter in &mut self.sprite_shifters {
                shifter.tick();
            }

            return None;
        }

        let shifters = &mut self.sprite_shifters;
        for (idx, shifter) in shifters.iter_mut().enumerate() {
            let pattern = shifter.tick();

            match ret {
                Some(_) => continue,
                _ if pattern == 0 => continue,
                None => {
                    let attr = shifter.palette();
                    let priority = shifter.priority();
                    let palette_addr = 0x3F10 | (attr << 2) as u16 | pattern as u16;

                    let color = self.vram.loadb(palette_addr);
                    let color = Rgb::from_byte(color);

                    ret = Some((color, priority, idx));
                }
            }
        }

        ret
    }

    fn background_pixel(&mut self, px: u16) -> Option<&'static Rgb> {
        if px < 8 && self.ppumask.background_rendering_enabled() {
            return None;
        }

        let fine_x = self.vtwx.fine_x();
        let pattern = self.shifter.pattern_val(fine_x);

        if pattern == 0 {
            None
        } else {
            let attr = self.shifter.attr_val(fine_x);
            let palette_addr = 0x3F00 | (attr << 2) as u16 | pattern as u16;
            let color = self.vram.loadb(palette_addr);

            Some(Rgb::from_byte(color))
        }
    }

    fn global_bg(&mut self) -> &'static Rgb {
        let color = self.vram.loadb(0x3F00);
        Rgb::from_byte(color)
    }

    fn output_pixel(&mut self, line: u16, px: u16) {
        let sprite_pixel = self.sprite_pixel(px);
        let bg_pixel = self.background_pixel(px);

        let color = match (bg_pixel, sprite_pixel) {
            (None, None) => self.global_bg(),
            (Some(_), Some((p, SpritePriority::AboveBackground, idx))) => {
                // set sprite zero
                if idx == 0 && line != 255 {
                    self.ppustatus.set_sprite_zero_hit(true);
                }

                p
            }
            (None, Some((p, SpritePriority::AboveBackground, _))) => p,
            (None, Some((p, SpritePriority::BelowBackground, _))) => p,
            (Some(p), Some((_, _, idx))) => {
                // set sprite zero
                if idx == 0 && line != 255 {
                    self.ppustatus.set_sprite_zero_hit(true);
                }

                p
            }
            (Some(p), None) => p,
        };

        let r = line as usize;
        let c = px as usize;
        let base_output_addr = (r * 256 * 3) + (c * 3);

        self.screen[base_output_addr] = color.r;
        self.screen[base_output_addr + 1] = color.g;
        self.screen[base_output_addr + 2] = color.b;
    }
}

impl Mem for Ppu {
    fn peekb(&self, addr: u16) -> u8 {
        match addr & 0x07 {
            0x00 => *self.ppuctrl,
            0x01 => *self.ppumask,
            0x02 => self.ppustatus.get(),
            0x03 => self.oam.addr() as u8,
            0x04 => self.oam.peekb(self.oam.addr()),
            0x05 => 0,
            0x06 => 0,
            0x07 => 0,
            _ => unreachable!(),
        }
    }

    fn loadb(&mut self, addr: u16) -> u8 {
        match addr & 0x07 {
            0x00 => {
                warn!("Read from PPUCTRL");
                0
            }
            0x01 => {
                warn!("Read from PPUMASK");
                0
            }
            0x02 => {
                let ret = self.ppustatus.get();
                self.ppustatus.set_vblank_started(false);
                self.vtwx.reset_latch();
                ret
            }
            0x03 => {
                warn!("Read from OAMADDR");
                0
            }
            0x04 => {
                let addr = self.oam.addr();
                self.oam.loadb(addr)
            }
            0x05 => {
                warn!("Read from PPUSCROLL");
                0
            }
            0x06 => {
                warn!("Read from PPUADDR");
                0
            }
            0x07 => {
                let addr = self.vtwx.addr();
                let increment = self.ppuctrl.vram_addr_increment();

                if addr <= 0x3EFF {
                    let ret = self.ppu_data_buffer;
                    self.ppu_data_buffer = self.vram.loadb(addr);
                    self.vtwx.increment_addr(increment);

                    ret
                } else {
                    self.ppu_data_buffer = self.vram.loadb(addr);
                    self.vtwx.increment_addr(increment);
                    self.ppu_data_buffer
                }
            }
            _ => unreachable!(),
        }
    }

    fn storeb(&mut self, addr: u16, val: u8) {
        self.ppustatus.set_last_ppu_write(val);

        match addr & 0x07 {
            0x00 => {
                if !self.pos.is_warmed_up() {
                    return;
                }

                if bit!(val, 5) {
                    panic!("Sprite size set to 8x16");
                }

                if bit!(val, 6) {
                    panic!("PPU secondary mode selected");
                }

                if bit!(val, 7) && self.ppustatus.vblank_started() && self.pos.is_in_vblank() {
                    self.immediate_nmi = true;
                }

                self.ppuctrl.set(val);
                self.vtwx.store_ctrl(val);
            }
            0x01 => self.ppumask.set(val),
            0x02 => {}
            0x03 => self.oam.set_addr(val),
            0x04 => {
                let addr = self.oam.addr();
                self.oam.storeb(addr, val);
                self.oam.increment_addr();
            }
            0x05 => self.vtwx.store_scroll(val),
            0x06 => self.vtwx.store_addr(val),
            0x07 => {
                let inc = self.ppuctrl.vram_addr_increment();
                let addr = self.vtwx.ppu_addr();

                self.vram.storeb(addr, val);
                self.vtwx.increment_addr(inc);
            }
            _ => unreachable!(),
        }
    }
}
