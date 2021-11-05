use imgui::{Image, MenuItem, StyleColor, StyleVar, Ui, Window, im_str};
use nes::FrameBuffer;
use super::texture::Texture;
use super::View;

const SCREEN_WIDTH: f32 = 256.0;
const SCREEN_HEIGHT: f32 = 240.0;

pub struct NesView {
    attr_overlay: bool,
    frame_no: usize,
    last_frame: [u8; 256 * 240 * 3],
    tex: Texture,
}

impl NesView {
    pub fn new() -> Self {
        let tex = Texture::new();
        let frame_no = 0;
        let last_frame = [ 0; 256 * 240 * 3 ];
        let attr_overlay = false;

        Self { attr_overlay, frame_no, last_frame, tex }
    }

    /// Scale the given source height and width to fill the given bounds while maintaining
    /// the original aspect ratio
    fn scale(src_height: f32, src_width: f32, bounds: [f32; 2]) -> [f32; 2 ] {
        let width_ratio = bounds[0] / src_width;
        let height_ratio = bounds[1] / src_height;

        if width_ratio > height_ratio {
            let height = bounds[1];
            let width = src_width * height_ratio;
            [ height, width ]
        } else {
            let height = src_height * width_ratio;
            let width = bounds[0];
            [ height, width ]
        }
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

            self.tex.update(SCREEN_WIDTH as _, SCREEN_HEIGHT as _, &tmp_frame);
        } else {
            self.tex.update(SCREEN_WIDTH as _, SCREEN_HEIGHT as _, &self.last_frame);
        };
    }
}

impl View for NesView {
    fn window(&mut self, ui: &Ui) {
        let style = ui.push_style_var(StyleVar::WindowPadding([ 0.0, 0.0 ]));
        let bg_black = ui.push_style_color(StyleColor::WindowBg, [ 0.0, 0.0, 0.0, 1.0 ]);

        Window::new(im_str!("NES"))
            .collapsible(false)
            .size_constraints([ SCREEN_WIDTH, SCREEN_HEIGHT ], [ f32::MAX, f32::MAX ])
            .build(&ui, || {
                let win_size = ui.window_size();
                let img_size = NesView::scale(SCREEN_WIDTH, SCREEN_HEIGHT, win_size);

                // Center the image
                let img_left = (win_size[0] - img_size[0]) / 2.0;
                let img_top = (win_size[1] - img_size[1]) / 2.0;
                ui.set_cursor_pos([ img_left, img_top ]);

                Image::new(self.tex.id(), img_size).build(ui);
            });

        style.pop(&ui);
        bg_black.pop(&ui);
    }

    fn custom_menu(&mut self, ui: &Ui) {
        ui.menu(im_str!("Picture"), true, || {
            if MenuItem::new(im_str!("Attribute Table Grid"))
                .selected(self.attr_overlay)
                .build(&ui) {
                    self.attr_overlay = !self.attr_overlay;
                    self.repaint();
                }
        });
    }

    fn on_frame(&mut self, framebuffer: &mut FrameBuffer) {
        self.frame_no += 1;

        self.last_frame.copy_from_slice(framebuffer.frame());
        self.repaint();
    }
}
