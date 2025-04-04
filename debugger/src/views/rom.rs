use super::View;
use egui::{os::OperatingSystem, Button, Checkbox, Grid, KeyboardShortcut, ModifierNames};
use nes::INesHeader;
use std::path::PathBuf;

const SHORTCUT: KeyboardShortcut = shortcut!(ALT, R);

pub struct RomView {
    header: INesHeader,
    path: PathBuf,
    opened: bool,
}

impl RomView {
    pub fn new<P: Into<PathBuf>>(path: P, header: INesHeader) -> Self {
        let path = path.into();
        let opened = false;
        Self {
            header,
            path,
            opened,
        }
    }
}

impl View for RomView {
    fn window(&mut self, ctx: &egui::Context) {
        let opened = &mut self.opened;
        let path = &self.path;
        let header = &self.header;

        egui::Window::new("ROM Info")
            .open(opened)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(path.file_name().unwrap().to_str().unwrap());
                ui.separator();

                Grid::new("rom-info")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Region");
                        ui.label(format!("{:?}", header.region()));
                        ui.end_row();

                        ui.label("iNES version");
                        ui.label(format!("{}", header.ines_version()));
                        ui.end_row();

                        ui.label("Mapper");
                        ui.label(format!("{}", header.mapper()));
                        ui.end_row();

                        ui.label("Nametable mirroring");
                        ui.label(format!("{:?}", header.mirroring()));
                        ui.end_row();

                        ui.label("Battery save enabled");
                        let mut checked = header.has_battery_save();
                        let cb = Checkbox::without_text(&mut checked);
                        ui.add_enabled(false, cb);
                        ui.end_row();

                        ui.label("Has trainer");
                        let mut checked = header.has_trainer();
                        let cb = Checkbox::without_text(&mut checked);
                        ui.add_enabled(false, cb);
                        ui.end_row();

                        ui.label("PRG ROM banks");
                        ui.label(format!(
                            "{} ({}KB)",
                            header.prg_banks(),
                            header.prg_banks() * 16
                        ));
                        ui.end_row();

                        ui.label("CHR ROM banks");
                        ui.label(format!(
                            "{} ({}KB)",
                            header.chr_banks(),
                            header.chr_banks() * 8
                        ));
                        ui.end_row();
                    });
            });
    }

    fn main_menu(&mut self, ui: &mut egui::Ui) {
        let button = Button::new("ROM Info").shortcut_text(SHORTCUT.format(
            &ModifierNames::NAMES,
            OperatingSystem::from_target_os() == OperatingSystem::Mac,
        ));

        ui.add(button);
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.opened = !self.opened;
        }
    }
}
