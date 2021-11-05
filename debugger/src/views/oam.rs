use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use imgui::{MenuItem, Ui, Window, im_str};
use nes::{Mem, Nes};
use super::View;

pub struct OamView {
    opened: bool,
}

impl OamView {
    pub fn new() -> Self {
        Self {
            opened: true,
        }
    }
}

impl View for OamView { }
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggle = MenuItem::new(im_str!("Sprite OAM"))
//             .shortcut(im_str!("Alt-O"))
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
//         let oam = ppu.oam();
//
//         Window::new(im_str!("OAM"))
//             .opened(&mut self.opened)
//             .size_constraints([ 480., 300. ], [ f32::MAX, f32::MAX ])
//             .build(&ui, || {
//                 ui.columns(16, im_str!("oam"), true);
//
//                 for c in 0..16 {
//                     for r in 0..16 {
//                         let addr = r * 16 + c;
//                         let val = oam.peekb(addr);
//                         let s = format!("{:02X}", val);
//
//                         if val == 0 {
//                             ui.text_disabled(s);
//                         } else {
//                             ui.text(s);
//                         }
//                     }
//
//                     ui.next_column();
//                 }
//             });
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::O, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
