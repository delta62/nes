use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use imgui::{MenuItem, Ui, Window, im_str};
use nes::Nes;
use super::View;

pub struct CpuView {
    opened: bool,
}

impl CpuView {
    pub fn new() -> Self {
        let opened = false;
        Self { opened }
    }
}

impl View for CpuView { }
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggled = MenuItem::new(im_str!("CPU"))
//             .selected(self.opened)
//             .shortcut(im_str!("Alt-C"))
//             .build(&ui);
//
//         if toggled {
//             self.opened = !self.opened;
//         }
//     }
//
//     fn window(&mut self, ui: &Ui, nes: &Nes, _state: &mut EmulationState) {
//         if !self.opened {
//             return;
//         }
//
//         let cpu = nes.cpu();
//
//         Window::new(im_str!("CPU"))
//             .always_auto_resize(true)
//             .opened(&mut self.opened)
//             .build(&ui, || {
//                 ui.label_text(im_str!("Cycle"), &im_str!("{}",       cpu.cy()));
//                 ui.label_text(im_str!("PC"),    &im_str!("0x{:04X}", cpu.pc()));
//                 ui.label_text(im_str!("A"),     &im_str!("0x{:02X}", cpu.a()));
//                 ui.label_text(im_str!("X"),     &im_str!("0x{:02X}", cpu.x()));
//                 ui.label_text(im_str!("Y"),     &im_str!("0x{:02X}", cpu.y()));
//                 ui.label_text(im_str!("S"),     &im_str!("0x{:02X}", cpu.s()));
//                 ui.label_text(im_str!("P"),     &im_str!("0x{:02X}", cpu.flags()));
//
//                 ui.spacing();
//                 ui.separator();
//                 ui.spacing();
//
//                 ui.columns(4, im_str!("cpuflags"), false);
//
//                 let mut flags: u8 = cpu.flags();
//                 ui.checkbox_flags(im_str!("C"), &mut flags, 0x01);
//                 ui.checkbox_flags(im_str!("B"), &mut flags, 0x10);
//                 ui.next_column();
//                 ui.checkbox_flags(im_str!("Z"), &mut flags, 0x02);
//                 ui.checkbox_flags(im_str!("-"), &mut flags, 0x20);
//                 ui.next_column();
//                 ui.checkbox_flags(im_str!("I"), &mut flags, 0x04);
//                 ui.checkbox_flags(im_str!("V"), &mut flags, 0x40);
//                 ui.next_column();
//                 ui.checkbox_flags(im_str!("D"), &mut flags, 0x08);
//                 ui.checkbox_flags(im_str!("N"), &mut flags, 0x80);
//             });
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::C, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
