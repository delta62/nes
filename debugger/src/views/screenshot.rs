use chrono::Local;
use glfw::{Key, WindowEvent};
use imgui::{Ui, MenuItem, im_str};
use nes::FrameBuffer;
use super::View;
use crate::macros::press_ctrl;
use std::io::BufWriter;
use std::path::Path;
use std::fs::File;

pub struct ScreenshotView {
    last_frame: [ u8; 0x2D000 ], // 256 * 240 * 3
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
    fn custom_menu(&mut self, ui: &Ui) {
        ui.menu(im_str!("Screenshot"), true, || {
            let screenshot = MenuItem::new(im_str!("Screenshot"))
                .shortcut(im_str!("Ctrl-P"))
                .build(&ui);

            if screenshot {
                self.screenshot();
            }
        });
    }

    fn on_frame(&mut self, framebuffer: &mut FrameBuffer) {
        self.last_frame.copy_from_slice(framebuffer.frame());
    }

    fn key_event(&mut self, event: &WindowEvent) {
        if press_ctrl(Key::P, event) {
            self.screenshot();
        }
    }
}
