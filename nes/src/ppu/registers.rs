use log::warn;
use std::ops::Deref;

#[derive(Default, Debug, Copy, Clone)]
pub struct PpuControl(u8);

impl PpuControl {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, val: u8) {
        self.0 = val;
    }

    pub fn base_nametable_addr(&self) -> u16 {
        match self.0 & 0x03 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        }
    }

    pub fn vram_addr_increment(&self) -> u16 {
        if bit!(self.0, 2) {
            32
        } else {
            1
        }
    }

    pub fn sprite_pattern_table_address(&self) -> u16 {
        if bit!(self.0, 3) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn background_pattern_table_address(&self) -> u16 {
        if bit!(self.0, 4) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn sprite_size(&self) -> usize {
        if bit!(self.0, 5) {
            16
        } else {
            8
        }
    }

    pub fn is_primary(&self) -> bool {
        !bit!(self.0, 6)
    }

    pub fn generate_nmi(&self) -> bool {
        bit!(self.0, 7)
    }
}

impl Deref for PpuControl {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct PpuMask(u8);

impl PpuMask {
    pub fn set(&mut self, val: u8) {
        if bit!(val, 0) {
            warn!("Greyscale used but not implemented");
        }

        if bit!(val, 5) {
            warn!("Emphasize red used but not implemented");
        }

        if bit!(val, 6) {
            warn!("Emphasize green used but not implemented");
        }

        if bit!(val, 7) {
            warn!("Emphasize blue used but not implemented");
        }

        self.0 = val;
    }

    pub fn greyscale_enabled(&self) -> bool {
        bit!(self.0, 0)
    }

    pub fn hide_first_bg_tile(&self) -> bool {
        !bit!(self.0, 1)
    }

    pub fn hide_first_sprite_tile(&self) -> bool {
        !bit!(self.0, 2)
    }

    pub fn background_rendering_enabled(&self) -> bool {
        bit!(self.0, 3)
    }

    pub fn sprite_rendering_enabled(&self) -> bool {
        bit!(self.0, 4)
    }

    pub fn rendering_enabled(&self) -> bool {
        self.background_rendering_enabled() || self.sprite_rendering_enabled()
    }

    pub fn emphasize_red(&self) -> bool {
        bit!(self.0, 5)
    }

    pub fn emphasize_green(&self) -> bool {
        bit!(self.0, 6)
    }

    pub fn emphasize_blue(&self) -> bool {
        bit!(self.0, 7)
    }
}

impl Deref for PpuMask {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct PpuStatus {
    val: u8,
    last_write: u8,
}

impl PpuStatus {
    pub fn set_last_ppu_write(&mut self, val: u8) {
        self.last_write = val;
    }

    pub fn set_sprite_overflow(&mut self, val: bool) {
        toggle_bit!(self.val, 5, val);
    }

    pub fn set_sprite_zero_hit(&mut self, val: bool) {
        toggle_bit!(self.val, 6, val);
    }

    pub fn set_vblank_started(&mut self, val: bool) {
        toggle_bit!(self.val, 7, val);
    }

    pub fn sprite_overflow(&self) -> bool {
        bit!(self.val, 5)
    }

    pub fn sprite_zero_hit(&self) -> bool {
        bit!(self.val, 6)
    }

    pub fn vblank_started(&self) -> bool {
        bit!(self.val, 7)
    }

    pub fn get(&self) -> u8 {
        self.val & 0xE0 | self.last_write & 0x1F
    }
}

#[derive(Default)]
struct WriteToggle(bool);

impl WriteToggle {
    fn toggle(&mut self) {
        self.0 = !self.0;
    }

    fn reset(&mut self) {
        self.0 = false;
    }

    fn is_first(&self) -> bool {
        !self.0
    }
}

pub struct Vtwx {
    /// Temporary VRAM address (15 bits)
    t: u16,
    /// VRAM address (15 bits)
    v: u16,
    /// First/second write toggle switch
    w: WriteToggle,
    /// Fine X scroll position (3 bits)
    x: u8,
}

impl Vtwx {
    pub fn new() -> Self {
        Self {
            t: 0,
            v: 0,
            w: Default::default(),
            x: 0,
        }
    }

    pub fn store_ctrl(&mut self, val: u8) {
        let nt = ((val & 0x03) as u16) << 10;
        self.t = self.t & 0x73FF | nt;
    }

    pub fn store_scroll(&mut self, val: u8) {
        if self.w.is_first() {
            let fine_x = val & 0x07;
            let course_x = (val >> 3) as u16;

            self.x = fine_x;
            self.t = self.t & 0xFFE0 | course_x;
        } else {
            let fine_y = ((val & 0x07) as u16) << 12;
            let course_y = ((val & 0xF8) as u16) << 2;

            self.t = self.t & 0x0C1F | fine_y | course_y;
        }

        self.w.toggle();
    }

    pub fn store_addr(&mut self, val: u8) {
        if self.w.is_first() {
            // Set the high byte of the address
            let addr_hi = ((val & 0x3F) as u16) << 8;
            self.t = self.t & 0x0FF | addr_hi;
        } else {
            // Set the low byte of the address and clear bit 7
            self.t = self.t & 0x7F00 | val as u16;
            self.v = self.t;
        }

        self.w.toggle();
    }

    pub fn reset_latch(&mut self) {
        self.w.reset();
    }

    pub fn addr(&self) -> u16 {
        self.v
    }

    pub fn increment_h(&mut self) {
        if self.v & 0x001F == 0x001F {
            self.v ^= 0x400;
            self.v &= 0x7FE0;
        } else {
            self.v += 1;
        }
    }

    pub fn increment_v(&mut self) {
        let fine_y = (self.v & 0x7000) >> 0x0C;
        let course_y = (self.v & 0x03E0) >> 0x05;

        if fine_y < 0x07 {
            self.v += 0x1000; // Add 1 to fine y
        } else {
            if course_y == 0x1D {
                // The Y position wraps before the max value is reached because
                // the end of each nametable contains attribute bytes rather than
                // more tile bytes. BUGGY: The course Y position can be set out
                // of bounds with a write >= 240 to $2005, which will trigger the
                // next if/else branch.
                self.v ^= 0x0800; // flip nt
                self.v &= 0x7C1F; // set course y to 0
            } else if course_y == 0x001F {
                // BUGGY: If course y is set incorrectly as per above, it will
                // wrap once it hits 31.
                self.v &= 0x7C1F; // set course y to 0
            } else {
                self.v += 0x0020; // Add 1 to course y
            }

            self.v &= 0x0FFF; // set fine y to 0
        }
    }

    pub fn increment_addr(&mut self, increment: u16) {
        self.v += increment;
    }

    pub fn copy_h(&mut self) {
        self.v = self.v & 0x7BE0 | self.t & 0x041F;
    }

    pub fn copy_v(&mut self) {
        self.v = self.v & 0x041F | self.t & 0x7BE0;
    }

    pub fn ppu_addr(&self) -> u16 {
        self.v
    }

    pub fn tile_addr(&self) -> u16 {
        self.v & 0x0FFF | 0x2000
    }

    pub fn attr_addr(&self) -> u16 {
        let at_base = 0x23C0;
        let nt_base = self.v & 0x0C00;
        let course_y_hi = (self.v >> 4) & 0x38;
        let course_x_hi = (self.v >> 2) & 0x07;

        at_base | nt_base | course_y_hi | course_x_hi
    }

    pub fn attr_quadrant(&self) -> u16 {
        let x = (self.v & 0x02) >> 1;
        let y = (self.v & 0x40) >> 5;

        x | y
    }

    pub fn fine_y(&self) -> u16 {
        (self.v & 0x7000) >> 12
    }

    pub fn fine_x(&self) -> u8 {
        self.x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod vtwx {
        use super::*;

        #[test]
        fn latch_set_to_first_write_on_creation() {
            let vtwx = Vtwx::new();
            assert!(vtwx.w.is_first());
        }

        #[test]
        fn store_nt_sets_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.store_ctrl(0x03);
            assert_eq!(vtwx.t, 0x0C00);
        }

        #[test]
        fn store_nt_doesnt_clobber_other_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x7FFF;
            vtwx.store_ctrl(0xFC);
            assert_eq!(vtwx.t, 0x73FF);
        }

        #[test]
        fn store_scroll_lo_sets_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.store_scroll(0xAA);

            assert_eq!(vtwx.x, 0x02);
            assert_eq!(vtwx.t, 0x15);
        }

        #[test]
        fn store_scroll_lo_doesnt_clobber_other_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.x = 0x07;
            vtwx.t = 0x7FFF;
            vtwx.store_scroll(0x55);

            assert_eq!(vtwx.x, 0x05);
            assert_eq!(vtwx.t, 0x7FEA);
        }

        #[test]
        fn store_scroll_lo_toggles_latch() {
            let mut vtwx = Vtwx::new();
            vtwx.store_scroll(0xFF);

            assert!(!vtwx.w.is_first());
        }

        #[test]
        fn store_scroll_hi_sets_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.w.toggle();
            vtwx.store_scroll(0xAA);

            assert_eq!(vtwx.x, 0x00);
            assert_eq!(vtwx.t, 0x22A0);
        }

        #[test]
        fn store_scroll_hi_doesnt_clobber_other_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.w.toggle();
            vtwx.t = 0x7FFF;
            vtwx.x = 0x07;
            vtwx.store_scroll(0x55);

            assert_eq!(vtwx.x, 0x07);
            assert_eq!(vtwx.t, 0x5D5F);
        }

        #[test]
        fn store_scroll_hi_toggles_latch() {
            let mut vtwx = Vtwx::new();
            vtwx.w.toggle();
            vtwx.store_scroll(0xFF);

            assert!(vtwx.w.is_first());
        }

        #[test]
        fn store_addr_lo_sets_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.store_addr(0xAA);

            assert_eq!(vtwx.t, 0x2A00);
        }

        #[test]
        fn store_addr_lo_clobbers_bit_14() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x4000;
            vtwx.store_addr(0xFF);

            assert_eq!(vtwx.t, 0x3F00);
        }

        #[test]
        fn store_addr_lo_doesnt_clobber_other_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x7FFF;
            vtwx.store_addr(0x55);

            assert_eq!(vtwx.t, 0x15FF);
        }

        #[test]
        fn store_addr_lo_toggles_latch() {
            let mut vtwx = Vtwx::new();
            vtwx.store_addr(0xFF);

            assert!(!vtwx.w.is_first());
        }

        #[test]
        fn store_addr_hi_sets_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.w.toggle();
            vtwx.store_addr(0xAA);

            assert_eq!(vtwx.t, 0x00AA);
        }

        #[test]
        fn store_addr_hi_doesnt_clobber_other_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x7F00;
            vtwx.w.toggle();
            vtwx.store_addr(0x55);

            assert_eq!(vtwx.t, 0x7F55);
        }

        #[test]
        fn store_addr_hi_copies_t_to_v() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x5500;
            vtwx.w.toggle();
            vtwx.store_addr(0xAA);

            assert_eq!(vtwx.v, 0x55AA);
        }

        #[test]
        fn store_addr_hi_toggles_latch() {
            let mut vtwx = Vtwx::new();
            vtwx.w.toggle();
            vtwx.store_addr(0x00);

            assert!(vtwx.w.is_first());
        }

        #[test]
        fn copy_h_sets_horizontal_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x5555;
            vtwx.copy_h();

            assert_eq!(vtwx.v, 0x0415);
        }

        #[test]
        fn copy_h_leaves_non_horizontal_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x5AAA;
            vtwx.t = 0x5555;
            vtwx.copy_h();

            assert_eq!(vtwx.v, 0x5EB5);
        }

        #[test]
        fn copy_v_sets_vertical_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.t = 0x5555;
            vtwx.copy_v();

            assert_eq!(vtwx.v, 0x5140);
        }

        #[test]
        fn copy_v_leaves_non_vertical_bits() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x5AAA;
            vtwx.t = 0x5555;
            vtwx.copy_v();

            assert_eq!(vtwx.v, 0x514A);
        }

        #[test]
        fn inc_h_increments_v_register() {
            let mut vtwx = Vtwx::new();
            vtwx.increment_h();

            assert_eq!(vtwx.v, 0x01);
        }

        #[test]
        fn inc_h_switches_nt_up() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x001F;
            vtwx.increment_h();

            assert_eq!(vtwx.v, 0x0400);
        }

        #[test]
        fn inc_h_switches_nt_down() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x041F;
            vtwx.increment_h();

            assert_eq!(vtwx.v, 0x0000);
        }

        #[test]
        fn inc_v_increments_fine_y() {
            let mut vtwx = Vtwx::new();
            vtwx.increment_v();

            assert_eq!(vtwx.v, 0x1000);
        }

        #[test]
        fn inc_v_increments_course_y() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x7000;
            vtwx.increment_v();

            assert_eq!(vtwx.v, 0x0020);
        }

        #[test]
        fn inc_v_switches_nt_up() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x73A0;
            vtwx.increment_v();

            assert_eq!(vtwx.v, 0x0800);
        }

        #[test]
        fn inc_v_switches_nt_down() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x7BA0;
            vtwx.increment_v();

            assert_eq!(vtwx.v, 0x0000);
        }

        #[test]
        fn inc_v_increments_glitchy() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x73C0;
            vtwx.increment_v();

            assert_eq!(vtwx.v, 0x03E0);
        }

        #[test]
        fn inc_v_force_wraps_course_y() {
            let mut vtwx = Vtwx::new();
            vtwx.v = 0x73E0;
            vtwx.increment_v();

            assert_eq!(vtwx.v, 0x0000);
        }
    }
}
