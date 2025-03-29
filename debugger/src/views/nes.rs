use std::sync::mpsc::Sender;

use super::View;
use egui::{Context, Ui};
use log::warn;
use nes::{ControlMessage, Frame};

const SCREEN_WIDTH: f32 = 256.0;
const SCREEN_HEIGHT: f32 = 240.0;

pub struct NesView {
    attr_overlay: bool,
    last_frame: Frame,
    texture: Option<egui::TextureHandle>,
}

impl NesView {
    pub fn new() -> Self {
        let last_frame = Frame::new();
        let attr_overlay = false;
        let texture = None;

        Self {
            attr_overlay,
            last_frame,
            texture,
        }
    }

    // fn repaint(&mut self) {
    //     if self.attr_overlay {
    //         // vertical line
    //         for r in 0..16 {
    //             for c in 0..240 {
    //                 let addr = r * 16 * 3 + c * 256 * 3;
    //                 self.last_frame[addr + 0] = 0x02;
    //                 self.last_frame[addr + 1] = 0x08;
    //                 self.last_frame[addr + 2] = 0xA0;
    //             }
    //         }

    //         // horizontal line
    //         for c in 0..15 {
    //             for r in 0..256 {
    //                 let addr = r * 3 + c * 256 * 3 * 16;
    //                 self.last_frame[addr + 0] = 0x02;
    //                 self.last_frame[addr + 1] = 0x08;
    //                 self.last_frame[addr + 2] = 0xA0;
    //             }
    //         }
    //     };
    // }
}

impl View for NesView {
    fn window(&mut self, ctx: &Context) {
        let t = &mut self.texture;
        let last_frame = self.last_frame.as_ref();

        let texture = t.get_or_insert_with(|| {
            let img = egui::ColorImage::from_rgb([256, 240], last_frame);
            ctx.load_texture("NES Screen", img, Default::default())
        });

        egui::Window::new("NES")
            .collapsible(false)
            .default_size([SCREEN_WIDTH, SCREEN_HEIGHT])
            .min_width(SCREEN_WIDTH)
            .min_height(SCREEN_HEIGHT)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::BLACK)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                ui.image((texture.id(), texture.size_vec2()));
            });
    }

    fn custom_menu(&mut self, ui: &mut Ui, _ctx: &Context) {
        ui.menu_button("Picture", |ui| {
            let button = egui::Button::new("Attribute Table Grid").selected(self.attr_overlay);

            if ui.add(button).clicked() {
                self.attr_overlay = !self.attr_overlay;
            }
        });
    }

    fn on_frame(&mut self, frame: &Frame, _ctrl: &Sender<ControlMessage>) {
        warn!("frame");
        self.last_frame = frame.clone();
    }
}
