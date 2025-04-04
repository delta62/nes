use super::View;
use egui::{
    os::OperatingSystem, Button, Color32, Context, Frame, Grid, KeyboardShortcut, ModifierNames,
    Window,
};
use nes::{ControlMessage, ControlRequest, ControlResponse, EmulationState, PaletteState};
use std::sync::mpsc::Sender;

const SHORTCUT: KeyboardShortcut = shortcut!(ALT, A);

pub struct PaletteView {
    opened: bool,
    palette: PaletteState,
    state: EmulationState,
}

impl PaletteView {
    pub fn new(state: EmulationState) -> Self {
        let opened = false;
        let palette = Default::default();

        Self {
            opened,
            palette,
            state,
        }
    }
}

impl View for PaletteView {
    fn init(&mut self, _ctx: &Context, control: &Sender<ControlMessage>) {
        let req = ControlRequest::PaletteState;
        let _ = control.send(ControlMessage::ControlRequest(req));
    }

    fn window(&mut self, ctx: &Context) {
        let opened = &mut self.opened;
        let palette = &self.palette;

        Window::new("Palette")
            .open(opened)
            .collapsible(false)
            .fixed_size([275.0, 300.0])
            .show(ctx, |ui| {
                ui.label("Background");

                Grid::new("palette-bg").show(ui, |ui| {
                    if palette.background.len() < 16 {
                        return;
                    }

                    for i in 0..4 {
                        ui.label(format!("0x{:04X}", 0x3f00 + i * 4));
                        for j in 0..4 {
                            let rgb = palette.background[i * 4 + j];
                            let frame = Frame::NONE
                                .fill(Color32::from_rgb(rgb.r, rgb.g, rgb.b))
                                .stroke((1.0, Color32::WHITE))
                                .inner_margin(12.0);
                            frame.show(ui, |_| {});
                        }
                        ui.end_row();
                    }
                });

                ui.separator();

                ui.label("Sprites");
                Grid::new("palette-fg").show(ui, |ui| {
                    if palette.sprites.len() < 16 {
                        return;
                    }

                    for i in 0..4 {
                        ui.label(format!("0x{:04X}", 0x3f00 + i * 4));
                        for j in 0..4 {
                            let rgb = palette.sprites[i * 4 + j];
                            let frame = Frame::NONE
                                .fill(Color32::from_rgb(rgb.r, rgb.g, rgb.b))
                                .stroke((1.0, Color32::WHITE))
                                .inner_margin(12.0);
                            frame.show(ui, |_| {});
                        }
                        ui.end_row();
                    }
                });
            });
    }

    fn on_state_change(&mut self, state: EmulationState) {
        self.state = state;
    }

    fn on_step(&mut self, _log: &str, ctrl: &Sender<nes::ControlMessage>) {
        if let EmulationState::Run(_) = self.state {
            return;
        }

        let req = ControlRequest::PaletteState;
        let _ = ctrl.send(ControlMessage::ControlRequest(req));
    }

    fn on_control_response(&mut self, message: &ControlResponse) {
        if let ControlResponse::PaletteState(state) = message {
            self.palette = state.clone();
        }
    }

    fn main_menu(&mut self, ui: &mut egui::Ui) {
        let btn = Button::new("Palette").shortcut_text(SHORTCUT.format(
            &ModifierNames::NAMES,
            OperatingSystem::from_target_os() == OperatingSystem::Mac,
        ));
        if ui.add(btn).clicked() {
            self.opened = !self.opened;
        }
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.opened = !self.opened;
        }
    }
}

//     fn on_step(&mut self, nes: &Nes) {
//         if !self.needs_update {
//             return;
//         }
//
//         self.needs_update = false;
//
//         let ppu = nes.ppu().borrow();
//         let vram = ppu.vram();
//
//         for x in 0..16 {
//             let addr = 0x3F00 + x;
//             let val = vram.peekb(addr);
//             let rgb = Rgb::from_byte(val);
//
//             self.textures[x as usize].update(1, 1, &[rgb.r, rgb.g, rgb.b]);
//         }
//
//         for x in 0..16 {
//             let addr = 0x3F10 + x;
//             let val = vram.peekb(addr);
//             let rgb = Rgb::from_byte(val);
//
//             self.textures[x as usize + 16].update(1, 1, &[rgb.r, rgb.g, rgb.b]);
//         }
//     }
// }
