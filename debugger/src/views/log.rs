use crate::logging::NesLogger;
use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use imgui::{ChildWindow, MenuItem, ListClipper, Ui, Window, im_str};
use nes::Nes;
use std::collections::VecDeque;
use super::View;

pub struct LogView {
    capacity: usize,
    logger: NesLogger,
    logs: VecDeque<String>,
    opened: bool,
    scroll_to_bottom: bool,
}

impl LogView {
    pub fn new() -> Self {
        let capacity = 256;
        let logger = NesLogger::new();
        let logs = VecDeque::with_capacity(capacity);
        let opened = false;
        let scroll_to_bottom = false;

        Self {
            capacity,
            logger,
            logs,
            opened,
            scroll_to_bottom,
        }
    }
}

impl View for LogView { }
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggled = MenuItem::new(im_str!("Logs"))
//             .shortcut(im_str!("Alt-L"))
//             .selected(self.opened)
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
//         let logs = &mut self.logs;
//         let scroll_to_bottom = self.scroll_to_bottom;
//         let capacity = self.capacity;
//
//         self.scroll_to_bottom = false;
//
//         Window::new(im_str!("Log"))
//             .opened(&mut self.opened)
//             .size_constraints([ 685.0, 140.0 ], [ f32::MAX, f32::MAX ])
//             .build(&ui, || {
//                 ui.text(format!("Showing last {} items", capacity));
//                 ui.same_line(180.0);
//                 if ui.small_button(im_str!("Clear")) {
//                     logs.clear();
//                 }
//
//                 ui.spacing();
//                 ui.separator();
//                 ui.spacing();
//
//                 ChildWindow::new(1)
//                     .build(ui, || {
//                         let mut clipper = ListClipper::new(logs.len() as i32).begin(ui);
//                         while clipper.step() {
//                             for line in clipper.display_start()..clipper.display_end() {
//                                 ui.text(&logs[line as usize]);
//                             }
//                         }
//
//                         if ui.scroll_y() > ui.scroll_max_y() {
//                             ui.set_scroll_here_y();
//                         }
//
//                         if scroll_to_bottom {
//                             let max = ui.scroll_max_y();
//                             ui.set_scroll_y(max);
//                         }
//                 });
//             });
//     }
//
//     fn on_step(&mut self, nes: &Nes) {
//         if !self.opened {
//             return;
//         }
//
//         if nes.cpu().is_busy() {
//             return;
//         }
//
//         if self.logs.len() >= self.capacity {
//             self.logs.pop_front();
//         }
//
//         let log = self.logger.log(nes);
//         self.logs.push_back(log);
//         self.scroll_to_bottom = true;
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::L, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
