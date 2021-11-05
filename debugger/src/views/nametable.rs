use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use imgui::{ChildWindow, MenuItem, Selectable, StyleVar, Ui, Window, im_str};
use nes::{Mem, Nes};
use super::View;

pub struct NametableView {
    opened: bool,
    base: u16,
}

impl NametableView {
    pub fn new() -> Self {
        Self {
            opened: false,
            base: 0x2000,
        }
    }
}

impl View for NametableView {}
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggle = MenuItem::new(im_str!("Nametables"))
//             .shortcut(im_str!("Alt-N"))
//             .selected(self.opened)
//             .build(&ui);
//
//         if toggle {
//             self.opened = !self.opened;
//         }
//     }
//
//     fn window(&mut self, ui: &Ui, nes: &Nes, _state: &mut EmulationState) {
//         if !self.opened {
//             return;
//         }
//
//         let ppu = nes.ppu().borrow();
//         let vram = ppu.vram();
//         let base = &mut self.base;
//
//         Window::new(im_str!("Nametables"))
//             .opened(&mut self.opened)
//             .size_constraints([ 875., 400. ], [ f32::MAX, f32::MAX ])
//             .build(&ui, || {
//                 let tok = ui.push_style_var(StyleVar::SelectableTextAlign([ 0.5, 0.5 ]));
//                 ui.columns(4, im_str!("nt_select"), false);
//                 if Selectable::new(im_str!("0x2000"))
//                     .selected(*base == 0x2000)
//                     .build(&ui) { *base = 0x2000 }
//                 ui.next_column();
//                 if Selectable::new(im_str!("0x2400"))
//                     .selected(*base == 0x2400)
//                     .build(&ui) { *base = 0x2400 }
//                 ui.next_column();
//                 if Selectable::new(im_str!("0x2800"))
//                     .selected(*base == 0x2800)
//                     .build(&ui) { *base = 0x2800 }
//                 ui.next_column();
//                 if Selectable::new(im_str!("0x2C00"))
//                     .selected(*base == 0x2C00)
//                     .build(&ui) { *base = 0x2C00 }
//
//                 tok.pop(ui);
//                 ui.columns(1, im_str!("nt_table"), false);
//
//                 ui.spacing();
//                 ui.separator();
//                 ui.spacing();
//
//                 ChildWindow::new(1).build(ui, || {
//                     ui.columns(32, im_str!("nametable"), true);
//
//                     for c in 0..32 {
//                         for r in 0..30 {
//                             let addr = *base + r * 32 + c;
//                             let val = vram.peekb(addr);
//                             let s = format!("{:02X}", val);
//
//                             if val == 0 {
//                                 ui.text_disabled(s);
//                             } else {
//                                 ui.text(s);
//                             }
//                         }
//
//                         ui.next_column();
//                     }
//                 });
//             });
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::N, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
