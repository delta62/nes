use super::View;
use chrono::Local;
use egui::{os::OperatingSystem, Button, Context, KeyboardShortcut, ModifierNames, Ui};
use nes::{ControlMessage, Frame};
use std::{fs::File, io::BufWriter, path::Path, sync::mpsc::Sender};

const SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: egui::Modifiers::CTRL,
    logical_key: egui::Key::P,
};

pub struct ScreenshotView {
    shot_pending: bool,
}

impl ScreenshotView {
    pub fn new() -> Self {
        let shot_pending = false;
        Self { shot_pending }
    }
}

impl ScreenshotView {
    fn screenshot(&self, frame: &Frame) {
        let now = Local::now();
        let path = now.format("screenshot-%Y-%m-%d-%H-%M-%S.png").to_string();
        let path = Path::new(&path);
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, 256, 240);

        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(frame.as_ref()).unwrap();
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
                self.shot_pending = true;
            }
        });
    }

    fn on_frame(&mut self, frame: &Frame, _ctrl: &Sender<ControlMessage>) {
        if self.shot_pending {
            self.screenshot(frame);
            self.shot_pending = false;
        }
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.shot_pending = true;
        }
    }
}
