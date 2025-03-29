use super::View;
use egui::{os::OperatingSystem, Context, KeyboardShortcut, ModifierNames, Ui};
use nes::{ControlMessage, ControlRequest, ControlResponse, EmulationState, PpuState};
use std::sync::mpsc::Sender;

const SHORTCUT: KeyboardShortcut = shortcut!(ALT, P);

pub struct PpuView {
    state: EmulationState,
    ppu_state: PpuState,
    opened: bool,
}

impl PpuView {
    pub fn new(state: EmulationState) -> Self {
        let opened = false;
        let ppu_state = Default::default();
        Self {
            opened,
            state,
            ppu_state,
        }
    }
}

impl View for PpuView {
    fn init(&mut self, control: &Sender<nes::ControlMessage>) {
        let msg = ControlRequest::PpuState;
        let msg = ControlMessage::ControlRequest(msg);
        let _ = control.send(msg);
    }

    fn on_state_change(&mut self, state: EmulationState) {
        self.state = state;
    }

    fn on_control_response(&mut self, message: &nes::ControlResponse) {
        if let ControlResponse::PpuState(state) = message {
            self.ppu_state = state.clone();
        }
    }

    fn main_menu(&mut self, ui: &mut Ui) {
        let button = egui::Button::new("PPU")
            .selected(self.opened)
            .shortcut_text(SHORTCUT.format(
                &ModifierNames::NAMES,
                OperatingSystem::from_target_os() == OperatingSystem::Mac,
            ));

        if ui.add(button).clicked() {
            self.opened = !self.opened;
        }
    }

    fn window(&mut self, ctx: &Context) {
        if !self.opened {
            return;
        }

        egui::Window::new("PPU").auto_sized().show(ctx, |ui| {
            if self.state == EmulationState::Run {
                ui.label("Emulation running");
                return;
            }

            egui::Grid::new("ppugeneral").show(ui, |ui| {
                ui.label("Cycle");
                ui.label(format!("{}", self.ppu_state.cycle));
                ui.end_row();
                ui.label("Frame");
                ui.label(format!("{}", self.ppu_state.frame));
                ui.end_row();
                ui.label("Scanline");
                ui.label(format!("{}", self.ppu_state.scanline));
                ui.end_row();
                ui.label("Pixel");
                ui.label(format!("{}", self.ppu_state.pixel));
                ui.end_row();

                ui.separator();
                ui.end_row();

                ui.label("PPUCTRL");
                ui.label(format!("0x{:02x}", *self.ppu_state.ctrl));
                ui.end_row();
                ui.label("PPUMASK");
                ui.label(format!("0x{:02x}", *self.ppu_state.mask));
                ui.end_row();
                ui.label("PPUSTATUS");
                ui.label(format!("0x{:02x}", self.ppu_state.status.get()));
                ui.end_row();
                ui.label("OAMADDR");
                ui.label(format!("0x{:02x}", self.ppu_state.oamaddr));
                ui.end_row();
                ui.label("OAMDATA");
                ui.label(format!("0x{:02x}", self.ppu_state.oamdata));
                ui.end_row();
                ui.label("PPUADDR");
                ui.label(format!("0x{:02x}", self.ppu_state.addr));
                ui.end_row();
                ui.label("PPUDATA");
                ui.label(format!("0x{:02x}", self.ppu_state.data));
                ui.end_row();
                ui.label("OAMDMA");
                ui.label(format!("0x{:02x}", self.ppu_state.oamdma));
                ui.end_row();

                ui.separator();
                ui.end_row();
            });

            ui.collapsing("PPUCTRL", |ui| {
                let ctrl = self.ppu_state.ctrl;
                egui::Grid::new("ppuctrl").show(ui, |ui| {
                    ui.label("Base NT addr");
                    ui.label(format!("0x{:04x}", ctrl.base_nametable_addr()));
                    ui.end_row();
                    ui.label("VRAM inc");
                    ui.label(format!("{}", ctrl.vram_addr_increment()));
                    ui.end_row();
                    ui.label("Sprite addr");
                    ui.label(format!("{}", ctrl.sprite_pattern_table_address()));
                    ui.end_row();
                    ui.label("Backgr addr");
                    ui.label(format!("{}", ctrl.background_pattern_table_address()));
                    ui.end_row();
                    ui.label("Sprite size");
                    ui.label(format!("{}", ctrl.sprite_size()));
                    ui.end_row();
                    ui.label("Pri/sec sel");
                    ui.label(if ctrl.is_primary() {
                        "primary"
                    } else {
                        "secondary"
                    });
                    ui.end_row();
                    ui.label("NMI enabled");
                    ui.label(format!("{}", ctrl.generate_nmi()));
                    ui.end_row();
                });
            });

            ui.collapsing("PPUMASK", |ui| {
                let mask = self.ppu_state.mask;
                egui::Grid::new("ppumask").show(ui, |ui| {
                    ui.disable();
                    ui.checkbox(&mut mask.greyscale_enabled(), "Greyscale");
                    ui.end_row();
                    ui.checkbox(&mut mask.hide_first_bg_tile(), "Hide bg 0");
                    ui.end_row();
                    ui.checkbox(&mut mask.hide_first_sprite_tile(), "Hide fg 0");
                    ui.end_row();
                    ui.checkbox(&mut mask.background_rendering_enabled(), "BG enabled");
                    ui.end_row();
                    ui.checkbox(&mut mask.rendering_enabled(), "FG enabled");
                    ui.end_row();
                    ui.checkbox(&mut mask.emphasize_red(), "Emph. red");
                    ui.end_row();
                    ui.checkbox(&mut mask.emphasize_green(), "Emph. green");
                    ui.end_row();
                    ui.checkbox(&mut mask.emphasize_blue(), "Emph. blue");
                    ui.end_row();
                });
            });

            ui.collapsing("PPUSTATUS", |ui| {
                let status = self.ppu_state.status;
                egui::Grid::new("ppustatus").show(ui, |ui| {
                    ui.disable();
                    ui.checkbox(&mut status.sprite_overflow(), "Sprite ovflw");
                    ui.end_row();
                    ui.checkbox(&mut status.sprite_zero_hit(), "Sprite 0 hit");
                    ui.end_row();
                    ui.checkbox(&mut status.vblank_started(), "VBlank start");
                    ui.end_row();
                });
            });

            ui.collapsing("PPUSCROLL", |ui| {
                let state = &self.ppu_state;
                egui::Grid::new("ppuscroll").show(ui, |ui| {
                    ui.disable();
                    ui.label("Scroll X");
                    ui.label(format!("{}", state.scroll_x));
                    ui.end_row();
                    ui.label("Scroll Y");
                    ui.label(format!("{}", state.scroll_y));
                    ui.end_row();
                });
            });
        });
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.opened = !self.opened;
        }
    }
}
