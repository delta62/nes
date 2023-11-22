use chrono::Local;
use glfw::{Key, WindowEvent};
use imgui::{MenuItem, Ui, im_str};
use crate::macros::press_ctrl;
use nes::FrameBuffer;
use super::View;
// use scap::{Mp4Config, Mp4Encoder};

const AUDIO_BIT_RATE: i64 = 192_000;
const AUDIO_SAMPLE_RATE: i32 = 44_100;
const NES_PICTURE_WIDTH: i32 = 256;
const NES_PICTURE_HEIGHT: i32 = 240;
const VIDEO_BIT_RATE: i64 = 700_000;

pub struct RecordView {
    recorder: Option<()>,
}

impl RecordView {
    pub fn new(record: bool) -> Self {
        let recorder = None;
        let mut ret = Self { recorder };

        if record {
            ret.toggle_recording();
        }

        ret
    }

    fn toggle_recording(&mut self) {
        if self.recorder.is_some() {
            // let recorder = std::mem::take(&mut self.recorder);
            // recorder.unwrap().finish().unwrap();
        } else {
            let now = Local::now();
            let path = now.format("recording-%Y-%m-%dT%H%M%S.mp4").to_string();
            // let config = Mp4Config {
            //     input_width: NES_PICTURE_WIDTH,
            //     input_height: NES_PICTURE_HEIGHT,
            //     output_width: NES_PICTURE_WIDTH,
            //     output_height: NES_PICTURE_HEIGHT,
            //     video_bit_rate: VIDEO_BIT_RATE,
            //     audio_sample_rate: AUDIO_SAMPLE_RATE,
            //     audio_bit_rate: AUDIO_BIT_RATE,
            // };

            // let encoder = Mp4Encoder::new(path, &config).unwrap();
            // self.recorder = Some(encoder);
        }
    }
}

impl View for RecordView {
    fn custom_menu(&mut self, ui: &Ui) {
        ui.menu(im_str!("Recording"), true, || {
            let toggle = MenuItem::new(im_str!("Record Video"))
                .shortcut(im_str!("Ctrl-R"))
                .selected(self.recorder.is_some())
                .build(&ui);

            if toggle {
                self.toggle_recording();
            }
        });
    }

    fn on_frame(&mut self, framebuffer: &mut FrameBuffer) {
        if let Some(stream) = &mut self.recorder {
            let mut samples = framebuffer.take_audio_samples();
            let audio_samples = samples.make_contiguous();
            let frame = framebuffer.frame();

            // stream.encode_frame(&frame, audio_samples).unwrap();
        }
    }

    fn key_event(&mut self, event: &WindowEvent) {
        if press_ctrl(Key::R, event) {
            self.toggle_recording();
        }
    }

    fn destroy(&mut self) {
        if self.recorder.is_some() {
            self.toggle_recording();
        }
    }
}
