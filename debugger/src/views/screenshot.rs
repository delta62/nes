use chrono::Local;
use egui::{Button, Ui, KeyboardShortcut, os::OperatingSystem, ModifierNames};
use nes::FrameBuffer;
use super::View;
use std::io::BufWriter;
use std::path::Path;
use std::fs::File;

const SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: egui::Modifiers::CTRL,
    key: egui::Key::P,
};

pub struct ScreenshotView {
    last_frame: [ u8; 256 * 240 * 3 ], // 0x2D000 ], // 256 * 240 * 3
}

impl ScreenshotView {
    pub fn new() -> Self {
        Self {
            last_frame: [ 0; 0x2D000 ],
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
    fn custom_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Screenshot", |ui| {
            let button = Button::new("Screenshot")
                .shortcut_text(SHORTCUT.format(&ModifierNames::NAMES, OperatingSystem::from_target_os() == OperatingSystem::Mac));

            if ui.add(button).clicked() {
                self.screenshot();
            }
        });
    }

    fn on_frame(&mut self, framebuffer: &mut FrameBuffer) {
        self.last_frame.copy_from_slice(framebuffer.frame());
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.screenshot();
        }
    }
}
