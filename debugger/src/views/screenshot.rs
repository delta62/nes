use super::View;
use chrono::Local;
use egui::{os::OperatingSystem, Button, Context, KeyboardShortcut, ModifierNames, Ui};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: egui::Modifiers::CTRL,
    logical_key: egui::Key::P,
};

pub struct ScreenshotView {
    last_frame: [u8; 256 * 240 * 3], // 0x2D000 ], // 256 * 240 * 3
}

impl ScreenshotView {
    pub fn new() -> Self {
        Self {
            last_frame: [0; 0x2D000],
        }
    }
}

impl ScreenshotView {
    fn screenshot(&self) {
        let now = Local::now();
        let path = now.format("screenshot-%Y-%m-%d-%H-%M-%S.png").to_string();
        let path = Path::new(&path);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, 256, 240);

        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.last_frame).unwrap();
    }
}

impl View for ScreenshotView {
    fn custom_menu(&mut self, ui: &mut Ui, _ctx: &Context) {
        ui.menu_button("Screenshot", |ui| {
            let button = Button::new("Screenshot").shortcut_text(SHORTCUT.format(
                &ModifierNames::NAMES,
                OperatingSystem::from_target_os() == OperatingSystem::Mac,
            ));

            if ui.add(button).clicked() {
                self.screenshot();
            }
        });
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.screenshot();
        }
    }
}
