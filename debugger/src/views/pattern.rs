use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use super::texture::Texture;
use super::View;
use nes::{Mem, Nes};
use imgui::{ChildWindow, Image, MenuItem, Selectable, StyleVar, Ui, Window, im_str};

macro_rules! bitn {
    ($val:expr, $bit:expr) => { ($val & (1 << $bit)) >> $bit };
}

const TABLE_WIDTH: f32 = 128.0;
const TABLE_HEIGHT: f32 = 128.0;

pub struct PatternView {
    base: u16,
    needs_update: bool,
    opened: bool,
    tex: Texture,
    tex_mem0: [u8; 0x24000 ], // 16*16 tiles, 1 tile = 8x8 px, 1px = 3*u8
    tex_mem1: [u8; 0x24000 ], // 16*16 tiles, 1 tile = 8x8 px, 1px = 3*u8
}

impl PatternView {
    pub fn new() -> Self {
        let base = 0x0000;
        let needs_update = false;
        let opened = false;
        let tex = Texture::new();
        let tex_mem0 = [ 0; 0x24000 ];
        let tex_mem1 = [ 0; 0x24000 ];

        Self { base, needs_update, opened, tex, tex_mem0, tex_mem1 }
    }

    /// Scale the given source height and width to fill the given bounds while maintaining
    /// the original aspect ratio
    fn scale(src_height: f32, src_width: f32, bounds: [f32; 2]) -> [f32; 2 ] {
        let width_ratio = bounds[0] / src_width;
        let height_ratio = bounds[1] / src_height;

        if width_ratio > height_ratio {
            let height = bounds[1];
            let width = src_width * height_ratio;
            [ height, width ]
        } else {
            let height = src_height * width_ratio;
            let width = bounds[0];
            [ height, width ]
        }
    }

    fn update_pattern_table(&mut self, nes: &Nes, addr_base: u16, lo: bool) {
        let ppu = nes.ppu().borrow();
        let vram = ppu.vram();
        let mem = if lo { &mut self.tex_mem0 } else { &mut self.tex_mem1 };

        // 1 row = 16 tiles (8x8 tiles) = 128px * 3 (3 channels) = 384px per row in mem buffer

        for i in 0..256 {
            for row in 0..8 {
                let addr = addr_base + i * 16 + row;
                let pattern_lo = vram.peekb(addr);
                let pattern_hi = vram.peekb(addr + 8);

                for col in 0..8 {
                    let lo = bitn!(pattern_lo, 7 - col);
                    let hi = bitn!(pattern_hi, 7 - col) << 1;
                    let pattern = lo + hi;

                    debug_assert!(pattern < 4, "pattern has too many bits");

                    let base_tile_offset = i / 16 * 384 * 8 + i % 16 * 24;
                    let row_addr = base_tile_offset + row * 384;
                    let addr = (row_addr + col * 3) as usize;
                    let color = pattern * 75;

                    mem[addr + 0] = color;
                    mem[addr + 1] = color;
                    mem[addr + 2] = color;
                }
            }
        }
    }
}

impl View for PatternView { }
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggled = MenuItem::new(im_str!("Pattern Tables"))
//             .selected(self.opened)
//             .shortcut(im_str!("Alt-T"))
//             .build(&ui);
//
//         if toggled {
//             self.opened = !self.opened;
//         }
//     }
//
//     fn window(&mut self, ui: &Ui, _nes: &Nes, _state: &mut EmulationState) {
//         if !self.opened {
//             return;
//         }
//
//         let tex = &mut self.tex;
//         let base = &mut self.base;
//         let mem0 = &self.tex_mem0;
//         let mem1 = &self.tex_mem1;
//
//         Window::new(im_str!("Pattern Tables"))
//             .size_constraints([ 128.0, 128.0 ], [ f32::MAX, f32::MAX ])
//             .opened(&mut self.opened)
//             .build(&ui, || {
//                 let tok = ui.push_style_var(StyleVar::SelectableTextAlign([ 0.5, 0.5 ]));
//                 ui.columns(2, im_str!("pt_select"), false);
//                 if Selectable::new(im_str!("0x0000"))
//                     .selected(*base == 0x0000)
//                     .build(&ui) {
//                         *base = 0x0000;
//                         tex.update(128, 128, mem0);
//                     }
//                 ui.next_column();
//                 if Selectable::new(im_str!("0x1000"))
//                     .selected(*base == 0x1000)
//                     .build(&ui) {
//                         *base = 0x1000;
//                         tex.update(128, 128, mem1);
//                     }
//
//                 tok.pop(ui);
//                 ui.columns(1, im_str!("pt_img"), false);
//
//                 ui.spacing();
//                 ui.separator();
//                 ui.spacing();
//
//                 ChildWindow::new(1)
//                     .build(ui, || {
//                         let win_size = ui.window_size();
//                         let img_size = PatternView::scale(TABLE_WIDTH, TABLE_HEIGHT, win_size);
//
//                         let img_left = (win_size[0] - img_size[0]) / 2.0;
//                         let img_top = (win_size[1] - img_size[1]) / 2.0;
//                         ui.set_cursor_pos([ img_left, img_top ]);
//
//                         Image::new(tex.id(), img_size).build(ui);
//                     });
//             });
//     }
//
//     fn on_step(&mut self, nes: &Nes) {
//         if !self.needs_update {
//             return;
//         }
//
//         self.needs_update = false;
//
//         self.update_pattern_table(nes, 0x0000, true);
//         self.update_pattern_table(nes, 0x1000, false);
//
//         let mem = if self.base == 0x000 { &self.tex_mem0 } else { &self.tex_mem1 };
//         self.tex.update(128, 128, mem);
//     }
//
//
//     fn on_frame(&mut self, _frame: &[u8]) {
//         self.needs_update = true;
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::T, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
