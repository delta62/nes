use super::View;
use egui::{Color32, ColorImage, Context, CornerRadius, Image, Key, TextureOptions, Vec2};
use nes::{Buttons, ControlMessage, Frame};
use std::{
    collections::VecDeque,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};

const FPS_SAMPLE_SECONDS: f32 = 1.0;
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

pub struct NesView {
    texture: Option<egui::TextureHandle>,
    sender: Sender<ControlMessage>,
    fps_samples: VecDeque<Instant>,
}

impl NesView {
    pub fn new(sender: Sender<ControlMessage>) -> Self {
        let texture = None;
        let fps_samples = VecDeque::with_capacity(60);

        Self {
            sender,
            texture,
            fps_samples,
        }
    }

    fn calculate_fps(&self) -> f32 {
        self.fps_samples.len() as f32 / FPS_SAMPLE_SECONDS
    }
}

impl View for NesView {
    fn init(&mut self, ctx: &Context, _ctrl: &Sender<ControlMessage>) {
        let img = egui::ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::DARK_BLUE);
        self.texture = Some(ctx.load_texture("nes-picture", img, Default::default()));
    }

    fn window(&mut self, ctx: &Context) {
        let fps = self.calculate_fps();

        egui::Window::new("NES")
            .title_bar(false)
            .collapsible(false)
            .default_size([SCREEN_WIDTH as f32 * 2.0, SCREEN_HEIGHT as f32 * 2.0])
            .min_width(SCREEN_WIDTH as f32)
            .min_height(SCREEN_HEIGHT as f32)
            .frame(
                egui::Frame::new()
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                ui.monospace(format!("{:.02}", fps));
                if let Some(tex) = &self.texture {
                    let avail_width = ui.available_width();
                    let image = Image::new(tex)
                        .maintain_aspect_ratio(true)
                        .fit_to_exact_size(Vec2::new(avail_width, avail_width));
                    ui.add(image);
                }
            });
    }

    fn on_frame(&mut self, frame: &Frame, _ctrl: &Sender<ControlMessage>) {
        let now = Instant::now();
        self.fps_samples.push_back(now);

        while let Some(&first) = self.fps_samples.front() {
            if now.duration_since(first) <= Duration::from_secs(FPS_SAMPLE_SECONDS as u64) {
                break;
            }

            self.fps_samples.pop_front();
        }

        let image = ColorImage::from_rgb([SCREEN_WIDTH, SCREEN_HEIGHT], frame.as_ref());
        if let Some(tex) = self.texture.as_mut() {
            tex.set(image, TextureOptions::NEAREST);
        };
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        let mut buttons = Buttons::empty();

        if input_state.key_down(Key::ArrowUp) {
            buttons |= Buttons::UP;
        }
        if input_state.key_down(Key::ArrowDown) {
            buttons |= Buttons::DOWN;
        }
        if input_state.key_down(Key::ArrowLeft) {
            buttons |= Buttons::LEFT;
        }
        if input_state.key_down(Key::ArrowRight) {
            buttons |= Buttons::RIGHT;
        }

        if input_state.key_down(Key::Z) {
            buttons |= Buttons::A;
        }
        if input_state.key_down(Key::X) {
            buttons |= Buttons::B;
        }
        if input_state.key_down(Key::Backspace) {
            buttons |= Buttons::SELECT;
        }
        if input_state.key_down(Key::Enter) {
            buttons |= Buttons::START;
        }

        let msg = ControlMessage::ControllerInput {
            gamepad1: buttons,
            gamepad2: Buttons::empty(),
        };
        let _ = self.sender.send(msg);
    }
}
