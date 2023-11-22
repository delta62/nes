use egui::{Context, Ui};
use nes::FrameBuffer;
use super::View;

const SCREEN_WIDTH: f32 = 256.0;
const SCREEN_HEIGHT: f32 = 240.0;

pub struct NesView {
    attr_overlay: bool,
    last_frame: [u8; 256 * 240 * 3],
    texture: Option<egui::TextureHandle>,
}

impl NesView {
    pub fn new() -> Self {
        let last_frame = [ 0; 256 * 240 * 3 ];
        let attr_overlay = false;
        let texture = None;

        Self { attr_overlay, last_frame, texture }
    }

    fn repaint(&mut self) {
        if self.attr_overlay {
            let mut tmp_frame = [ 0; 256 * 240 * 3 ];
            tmp_frame.copy_from_slice(&self.last_frame);

            // vertical line
            for r in 0..16 {
                for c in 0..240 {
                    let addr = r * 16 * 3 + c * 256 * 3;
                    tmp_frame[addr + 0] = 0x02;
                    tmp_frame[addr + 1] = 0x08;
                    tmp_frame[addr + 2] = 0xA0;
                }
            }

            // horizontal line
            for c in 0..15 {
                for r in 0..256 {
                    let addr = r * 3 + c * 256 * 3 * 16;
                    tmp_frame[addr + 0] = 0x02;
                    tmp_frame[addr + 1] = 0x08;
                    tmp_frame[addr + 2] = 0xA0;
                }
            }
        };
    }
}

impl View for NesView {
    fn window(&mut self, ctx: &Context) {
        let last_frame = &self.last_frame;
        let t = &mut self.texture;

        let texture = t.get_or_insert_with(|| {
            let img = egui::ColorImage::from_rgb([ 256, 240 ], last_frame);
            ctx.load_texture(
                "NES Screen",
                img,
                Default::default())
        });

        egui::Window::new("NES")
            .collapsible(false)
            .default_size([ SCREEN_WIDTH, SCREEN_HEIGHT ])
            .min_width(SCREEN_WIDTH)
            .min_height(SCREEN_HEIGHT)
            .frame(egui::Frame::none()
                .fill(egui::Color32::BLACK)
                .inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                ui.image((texture.id(), texture.size_vec2()));
            });
    }

    fn custom_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Picture", |ui| {
            let button = egui::Button::new("Attribute Table Grid")
                .selected(self.attr_overlay);

            if ui.add(button).clicked() {
                self.attr_overlay = !self.attr_overlay;
            }
        });
    }

    fn on_frame(&mut self, framebuffer: &mut FrameBuffer) {
        self.last_frame.copy_from_slice(framebuffer.frame());
        self.repaint();
    }
}
