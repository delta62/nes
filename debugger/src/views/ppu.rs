use egui::{Context, KeyboardShortcut, os::OperatingSystem, ModifierNames, Ui};
use nes::{Mem, Nes};
use super::View;

const SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: egui::Modifiers::ALT,
    key: egui::Key::P,
};

pub struct PpuView {
    opened: bool,
}

impl PpuView {
    pub fn new() -> Self {
        let opened = false;
        Self { opened }
    }
}

impl View for PpuView {
    fn main_menu(&mut self, ui: &mut Ui) {
        let button = egui::Button::new("PPU")
            .selected(self.opened)
            .shortcut_text(SHORTCUT.format(&ModifierNames::NAMES, OperatingSystem::from_target_os() == OperatingSystem::Mac));

        if ui.add(button).clicked() {
            self.opened = !self.opened;
        }
    }

    fn window(&mut self, ctx: &Context) {
        if !self.opened {
            return
        }

        egui::Window::new("PPU")
            .auto_sized()
            .show(ctx, |ui| {
                egui::Grid::new("ppugeneral").show(ui, |ui| {
                    ui.label("Cycle");
                    ui.label("42");
                    ui.end_row();
                    ui.label("Frame");
                    ui.label("42");
                    ui.end_row();
                    ui.label("Scanline");
                    ui.label("42");
                    ui.end_row();
                    ui.label("Pixel");
                    ui.label("42");
                    ui.end_row();

                    ui.separator();
                    ui.end_row();

                    ui.label("PPUCTRL");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("PPUMASK");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("PPUSTATUS");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("OAMADDR");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("OAMDATA");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("PPUSCROLL");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("PPUADDR");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("PPUDATA");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();
                    ui.label("OAMDMA");
                    ui.label(format!("0x{:02x}", 42));
                    ui.end_row();

                    ui.separator();
                    ui.end_row();
                });

                ui.collapsing("PPUCTRL", |ui| {
                    egui::Grid::new("ppuctrl")
                        .show(ui, |ui| {
                            ui.label("Base NT addr");
                            ui.label(format!("0x{:04x}", 42));
                            ui.end_row();
                            ui.label("VRAM inc");
                            ui.label("42");
                            ui.end_row();
                            ui.label("Sprite addr");
                            ui.label("42");
                            ui.end_row();
                            ui.label("Backgr addr");
                            ui.label("42");
                            ui.end_row();
                            ui.label("Sprite size");
                            ui.label("8x16");
                            ui.end_row();
                            ui.label("Pri/sec sel");
                            ui.label("primary");
                            ui.end_row();
                            ui.label("NMI enabled");
                            ui.label("true");
                            ui.end_row();
                        });
                });

                ui.collapsing("PPUMASK", |ui| {
                    egui::Grid::new("ppumask")
                        .show(ui, |ui| {
                            ui.checkbox(&mut false, "Greyscale");
                            ui.end_row();
                            ui.checkbox(&mut false, "Hide bg 0");
                            ui.end_row();
                            ui.checkbox(&mut false, "Hide fg 0");
                            ui.end_row();
                            ui.checkbox(&mut true, "BG enabled");
                            ui.end_row();
                            ui.checkbox(&mut true, "FG enabled");
                            ui.end_row();
                            ui.checkbox(&mut false, "Emph. red");
                            ui.end_row();
                            ui.checkbox(&mut false, "Emph. green");
                            ui.end_row();
                            ui.checkbox(&mut false, "Emph. blue");
                            ui.end_row();
                        });
                });

                ui.collapsing("PPUSTATUS", |ui| {
                    egui::Grid::new("ppustatus")
                        .show(ui, |ui| {
                            ui.checkbox(&mut false, "Sprite ovflw");
                            ui.end_row();
                            ui.checkbox(&mut false, "Sprite 0 hit");
                            ui.end_row();
                            ui.checkbox(&mut false, "VBlank start");
                            ui.end_row();
                        });
                });

                ui.collapsing("PPUSCROLL", |ui| {
                    egui::Grid::new("ppuscroll")
                        .show(ui, |ui| {
                            ui.label("Scroll X");
                            ui.label("42");
                            ui.end_row();
                            ui.label("Scroll Y");
                            ui.label("42");
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
