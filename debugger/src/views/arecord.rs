use chrono::Local;
use imgui::{MenuItem, Ui, im_str};
use nes::FrameBuffer;
use super::View;
// use scap::{WavConfig, WavEncoder};

const SAMPLE_RATE: i32 = 44_100;

pub struct AudioRecordView {
    recorder: Option<()>,
}

impl AudioRecordView {
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
            let path = now.format("recording-%Y-%m-%dT%H%M%S.wav").to_string();
            // let config = WavConfig {
            //     sample_rate: SAMPLE_RATE,
            // };

            // let encoder = WavEncoder::new(path, &config).unwrap();
            // self.recorder = Some(encoder);
        }
    }
}

impl View for AudioRecordView {
    fn custom_menu(&mut self, ui: &Ui) {
        ui.menu(im_str!("Recording"), true, || {
            let toggle = MenuItem::new(im_str!("Record Audio"))
                // .shortcut(im_str!("Ctrl-R"))
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

            // stream.encode(&mut samples).unwrap();
        }
    }

    fn destroy(&mut self) {
        if self.recorder.is_some() {
            self.toggle_recording();
        }
    }
}
