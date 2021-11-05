use crate::macros::{press, press_alt};
use glfw::{Key, WindowEvent};
use imgui::{MenuItem, Ui, Window, im_str};
use nes::Nes;
use super::View;

pub struct DebugView {
    opened: bool,
}

impl DebugView {
    pub fn new() -> Self {
        let opened = false;
        Self { opened }
    }
}

impl View for DebugView { }
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggle = MenuItem::new(im_str!("Debug"))
//             .shortcut(im_str!("Alt-D"))
//             .selected(self.opened)
//             .build(&ui);
//
//         if toggle {
//             self.opened = !self.opened;
//         }
//     }
//
//     fn window(&mut self, ui: &Ui, _nes: &Nes, state: &mut EmulationState) {
//         if !self.opened {
//             return;
//         }
//
//         Window::new(im_str!("Debug"))
//             .opened(&mut self.opened)
//             .always_auto_resize(true)
//             .build(&ui, || {
//                 ui.text(im_str!("{:?}", state));
//
//                 ui.spacing();
//
//                 let step = ui.button(im_str!("Step"), [ 50.0, 22.0 ]);
//                 ui.same_line_with_spacing(0.0, 10.0);
//                 let run = ui.button(im_str!("Run"), [ 50.0, 22.0 ]);
//                 ui.same_line_with_spacing(0.0, 10.0);
//                 let pause = ui.button(im_str!("Pause"), [ 50.0, 22.0 ]);
//                 let instr = ui.button(im_str!("Instr"), [ 50.0, 22.0 ]);
//                 ui.same_line_with_spacing(0.0, 10.0);
//                 let line = ui.button(im_str!("Line"), [ 50.0, 22.0 ]);
//                 ui.same_line_with_spacing(0.0, 10.0);
//                 let frame = ui.button(im_str!("Frame"), [ 50.0, 22.0 ]);
//
//                 if run {
//                     *state = EmulationState::Running;
//                 }
//
//                 if pause {
//                     *state = EmulationState::Paused;
//                 }
//
//                 if step {
//                     *state = EmulationState::StepCpuOnce;
//                 }
//
//                 if instr {
//                     *state = EmulationState::CpuInstruction;
//                 }
//
//                 if line {
//                     *state = EmulationState::PpuLine;
//                 }
//
//                 if frame {
//                     *state = EmulationState::PpuFrame;
//                 }
//             });
//     }
//
//     fn custom_menu(&mut self, ui: &Ui, state: &mut EmulationState) {
//         ui.menu(im_str!("Emulation"), true, || {
//             let pause = MenuItem::new(im_str!("Pause"))
//                 .selected(*state == EmulationState::Paused)
//                 .shortcut(im_str!("P"))
//                 .build(&ui);
//
//             let run = MenuItem::new(im_str!("Run"))
//                 .selected(*state == EmulationState::Running)
//                 .shortcut(im_str!("R"))
//                 .build(&ui);
//
//             let instr = MenuItem::new(im_str!("Next Instr"))
//                 .shortcut(im_str!("I"))
//                 .build(&ui);
//
//             let step = MenuItem::new(im_str!("Step CPU"))
//                 .shortcut(im_str!("S"))
//                 .build(&ui);
//
//             let line = MenuItem::new(im_str!("Next Line"))
//                 .shortcut(im_str!("L"))
//                 .build(&ui);
//
//             let frame = MenuItem::new(im_str!("Next Frame"))
//                 .shortcut(im_str!("F"))
//                 .build(&ui);
//
//             if pause {
//                 *state = EmulationState::Paused;
//             }
//
//             if run {
//                 *state = EmulationState::Running;
//             }
//
//             if instr {
//                 *state = EmulationState::CpuInstruction;
//             }
//
//             if step {
//                 *state = EmulationState::StepCpuOnce;
//             }
//
//             if line {
//                 *state = EmulationState::PpuLine;
//             }
//
//             if frame {
//                 *state = EmulationState::PpuFrame;
//             }
//         });
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, state: &mut EmulationState) {
//         if press(Key::S, event) {
//             *state = EmulationState::StepCpuOnce;
//         }
//
//         if press(Key::I, event) {
//             *state = EmulationState::CpuInstruction;
//         }
//
//         if press(Key::R, event) {
//             *state = EmulationState::Running;
//         }
//
//         if press(Key::P, event) {
//             *state = EmulationState::Paused;
//         }
//
//         if press(Key::F, event) {
//             *state = EmulationState::PpuFrame;
//         }
//
//         if press(Key::P, event) {
//             *state = EmulationState::PpuLine;
//         }
//
//         if press_alt(Key::D, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
