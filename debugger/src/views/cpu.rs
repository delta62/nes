use egui::{Context, KeyboardShortcut, ModifierNames, os::OperatingSystem, Ui};
use nes::Nes;
use super::View;

const SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: egui::Modifiers::ALT,
    key: egui::Key::C,
};

pub struct CpuView {
    opened: bool,
}

impl CpuView {
    pub fn new() -> Self {
        let opened = false;
        Self { opened }
    }
}

impl View for CpuView {
    fn main_menu(&mut self, ui: &mut Ui) {
        let button = egui::Button::new("CPU")
            .selected(self.opened)
            .shortcut_text(SHORTCUT.format(&ModifierNames::NAMES, OperatingSystem::from_target_os() == OperatingSystem::Mac));

        if ui.add(button).clicked() {
            self.opened = !self.opened;
        }
    }

    fn window(&mut self, ctx: &Context) {
        if !self.opened {
            return;
        }

        egui::Window::new("CPU")
            .auto_sized()
            .show(ctx, |ui| {
                egui::Grid::new("cpuregs")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Cycle");
                        ui.label("42");
                        ui.end_row();
                        ui.label("PC").on_hover_text("Program Counter");
                        ui.label(format!("0x{:04x}", 42));
                        ui.end_row();
                        ui.label("A").on_hover_text("Accumulator");
                        ui.label(format!("0x{:02x}", 42));
                        ui.end_row();
                        ui.label("X").on_hover_text("X Register");
                        ui.label(format!("0x{:02x}", 42));
                        ui.end_row();
                        ui.label("Y").on_hover_text("Y Register");
                        ui.label(format!("0x{:02x}", 42));
                        ui.end_row();
                        ui.label("S").on_hover_text("Stack Pointer");
                        ui.label(format!("0x{:02x}", 42));
                        ui.end_row();
                        ui.label("P").on_hover_text("Flags");
                        ui.label(format!("0x{:02x}", 42));
                        ui.end_row();
                    });

                egui::Grid::new("cpuflags").show(ui, |ui| {
                    ui.checkbox(&mut true, "C").on_hover_text("Carry flag");
                    ui.checkbox(&mut true, "Z").on_hover_text("Zero flag");
                    ui.checkbox(&mut true, "I").on_hover_text("Interrupt disable flag");
                    ui.checkbox(&mut true, "D").on_hover_text("Decimal flag");
                    ui.end_row();

                    ui.checkbox(&mut true, "B").on_hover_text("Break flag");
                    ui.label("-");
                    ui.checkbox(&mut true, "V").on_hover_text("Overflow flag");
                    ui.checkbox(&mut true, "N").on_hover_text("Negative flag");
                    ui.end_row();
                });
            });
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.opened = !self.opened;
        }
    }
}
