use crate::input::{Button, ButtonState, InputState};
use crate::nes::Nes;
use crate::rom::Rom;
use dasp::{interpolate::linear::Linear, Signal};
use log::warn;
use pcm::{Device as AudioDevice, DeviceConfig as AudioDeviceConfig};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

const AUDIO_SAMPLE_RATE: u32 = 44_100;
const CPU_FREQUENCY: f64 = 1_789_773.0;
const FRAME_SIZE: usize = 256 * 240 * 3;
const MAX_FB_SAMPLES: usize = (AUDIO_SAMPLE_RATE / 60 * 3) as usize; // 3 frames of audio

#[derive(Debug)]
enum AudioMessage {
    NeedMoreData(usize),
}

#[derive(Debug)]
pub enum VideoMessage {
    FrameAvailable,
}

#[derive(Debug)]
pub enum ControlMessage {
    ControllerInput(Button, ButtonState),
    SetState(EmulationState),
}

#[derive(Debug)]
pub enum EmulationState {
    Kill,
    Pause,
    Run,
}

pub struct FrameBuffer {
    audio_samples: VecDeque<f32>,
    buff: [u8; FRAME_SIZE],
}

impl FrameBuffer {
    pub fn new() -> Self {
        let audio_samples = VecDeque::with_capacity(MAX_FB_SAMPLES);
        let buff = [0; FRAME_SIZE];

        Self { audio_samples, buff }
    }

    fn add_audio_sample(&mut self, sample: f32) {
        if self.audio_samples.len() < MAX_FB_SAMPLES {
            self.audio_samples.push_back(sample);
        }
    }

    pub fn take_audio_samples(&mut self) -> VecDeque<f32> {
        let new_samples = VecDeque::with_capacity(MAX_FB_SAMPLES);

        std::mem::replace(&mut self.audio_samples, new_samples)
    }

    pub fn frame(&self) -> &[u8] {
        &self.buff
    }

    fn update(&mut self, frame: &[u8]) {
        self.buff.copy_from_slice(frame);
    }
}

struct AudioThread;

impl AudioThread {
    fn run(
        frame_buffer: Arc<Mutex<FrameBuffer>>,
        sample_buffer: Arc<Mutex<VecDeque<f32>>>,
        sender: Sender<AudioMessage>,
    ) {
        thread::spawn(|| {
            let config = AudioDeviceConfig {
                sample_rate: AUDIO_SAMPLE_RATE,
                channels: 1,
                buffer_target_us: 42_000,
                period_target_us: 8_000,
            };

            let audio_device = AudioDevice::with_config(config)
                .expect("Unable to start audio playback");

            audio_device.run(move |queue, wanted| {
                let mut samples = sample_buffer.lock().unwrap();
                let mut fb = frame_buffer.lock().unwrap();

                for _ in 0..wanted {
                    let sample = match samples.pop_front() {
                        Some(s) => s,
                        None => {
                            warn!("Not enough audio left in buffer");
                            0.0
                        }
                    };

                    queue.push_back(sample);
                    fb.add_audio_sample(sample);
                }

                sender
                    .send(AudioMessage::NeedMoreData(wanted))
                    .expect("Audio receiver hung up!");
            });
        });
    }
}

pub struct Emulation {
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    on_control: Receiver<ControlMessage>,
    on_frame: Sender<VideoMessage>,
    state: EmulationState,
    input_state: InputState,

    last1: f32,
    last2: f32,
}

impl Emulation {
    pub fn new(
        frame_buffer: Arc<Mutex<FrameBuffer>>,
        on_control: Receiver<ControlMessage>,
        on_frame: Sender<VideoMessage>,
    ) -> Self {
        let state = EmulationState::Pause;
        let input_state = InputState::default();

        let sample_buffer: VecDeque<f32> = VecDeque::with_capacity(8_000);
        let sample_buffer = Arc::new(Mutex::new(sample_buffer));

        let last1 = 0.0;
        let last2 = 0.0;

        Self {
            frame_buffer,
            input_state,
            last1,
            last2,
            on_control,
            on_frame,
            sample_buffer,
            state,
        }
    }

    pub fn run(mut self, rom: Rom) {
        let mut nes = Nes::with_rom(rom);
        let (audio_tx, mut audio_rx) = channel();

        let audio_buf = self.sample_buffer.clone();
        let audio_fb = self.frame_buffer.clone();
        AudioThread::run(audio_fb, audio_buf, audio_tx);

        loop {
            self.process_events();

            if let EmulationState::Kill = self.state {
                break;
            }

            self.produce_audio(&mut audio_rx, &mut nes);
        }
    }

    fn produce_audio(&mut self, audio_rx: &mut Receiver<AudioMessage>, nes: &mut Nes) {
        let message = audio_rx.recv().expect("Audio thread hung up!");
        match message {
            AudioMessage::NeedMoreData(wanted) => {

                let state = &self.state;
                let input_state = &self.input_state;
                let frame_buffer = &mut self.frame_buffer;
                let on_frame = &mut self.on_frame;

                let signal = dasp::signal::gen_mut(|| {
                    match state {
                        EmulationState::Pause | EmulationState::Kill => 0.0,
                        EmulationState::Run => {
                            let step_result = nes.step(input_state);
                            let sample = nes.apu().borrow().sample();

                            if step_result.new_frame {
                                let ppu = nes.ppu().borrow();
                                let mut fb = frame_buffer
                                    .lock()
                                    .expect("Unable to lock framebuffer");

                                fb.update(ppu.screen().as_ref());

                                on_frame
                                    .send(VideoMessage::FrameAvailable)
                                    .expect("Video thread hung up!");
                            }

                            sample
                        }
                    }
                });

                let linear = Linear::new(self.last1, self.last2);
                let mut signal = signal.from_hz_to_hz(
                    linear,
                    CPU_FREQUENCY,
                    AUDIO_SAMPLE_RATE as f64
                );
                let mut sample_buffer = self.sample_buffer.lock().unwrap();

                for _ in 0..wanted {
                    let sample = signal.next();
                    self.last1 = self.last2;
                    self.last2 = sample;
                    sample_buffer.push_back(sample);
                }
            }
        }
    }

    fn process_events(&mut self) {
        for message in self.on_control.try_iter() {
            match message {
                ControlMessage::SetState(state) => self.state = state,
                ControlMessage::ControllerInput(button, state) => {
                    self.input_state.gamepad1.button_press(button, state);
                }
            }
        }
    }
}
