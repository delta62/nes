use crate::cpu::Flags;
use crate::frame_buffer::{Frame, FrameBuffer};
use crate::input::{ButtonState, Buttons};
use crate::mem::Mem;
use crate::nes::Nes;
use crate::ppu::{PpuControl, PpuMask, PpuStatus};
use crate::rom::Rom;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SampleFormat, SampleRate, Stream, StreamConfig, SupportedStreamConfigRange};
use dasp::{interpolate::linear::Linear, Signal};
use std::mem;
use std::sync::mpsc::{Receiver, Sender};

const AUDIO_SAMPLE_RATE: u32 = 44_100;
const CPU_FREQUENCY: f64 = 1_789_773.0;

#[derive(Debug)]
pub enum ControlRequest {
    RegisterState,
    PpuState,
}

#[derive(Default, Clone, Debug)]
pub struct PpuState {
    // Picture state
    pub cycle: u64,
    pub frame: u64,
    pub scanline: u16,
    pub pixel: u16,

    // Registers
    pub ctrl: PpuControl,
    pub mask: PpuMask,
    pub status: PpuStatus,
    pub oamaddr: u16,
    pub oamdata: u8,
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub addr: u8,
    pub data: u8,
    pub oamdma: u8,
}

#[derive(Default, Debug, Clone)]
pub struct RegisterState {
    pub cycle: u64,
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: Flags,
}

#[derive(Debug)]
pub enum ControlResponse {
    RegisterState(RegisterState),
    PpuState(PpuState),
}

#[derive(Debug)]
pub enum VideoMessage {
    ControlResponse(ControlResponse),
    FrameAvailable(Frame),
    StateChanged(EmulationState),
}

#[derive(Debug)]
pub enum ControlMessage {
    ControllerInput { button: Buttons, state: ButtonState },
    SetState(EmulationState),
    RecycleFrame(Frame),
    ControlRequest(ControlRequest),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum EmulationState {
    /// The emulation has been ended by the user
    Kill,

    /// The emulation should step forward by one CPU tick, then transition into the Pause state
    Step,

    /// The emulation should remain paused forever
    Pause,

    /// The emulation should run uninterrupted until a user changes the state
    Run,
}

fn handle_control_message(
    message: ControlMessage,
    nes: &mut Nes,
    state: &mut EmulationState,
    on_frame: &Sender<VideoMessage>,
    frame_buffer: &mut FrameBuffer,
) {
    println!("{:?}", message);
    match message {
        ControlMessage::SetState(s) => {
            if s != *state {
                *state = s;
                let msg = VideoMessage::StateChanged(s);
                let _ = on_frame.send(msg);
            }
        }
        ControlMessage::ControllerInput { state, button } => match state {
            ButtonState::Pressed => nes.cpu.mem.input.press(button),
            ButtonState::Release => nes.cpu.mem.input.release(button),
        },
        ControlMessage::RecycleFrame(frame) => frame_buffer.put(frame),
        ControlMessage::ControlRequest(req) => match req {
            ControlRequest::RegisterState => {
                let state = RegisterState {
                    a: nes.cpu.a,
                    x: nes.cpu.x,
                    y: nes.cpu.y,
                    s: nes.cpu.s,
                    cycle: nes.cpu.cy,
                    p: nes.cpu.flags,
                    pc: nes.cpu.pc,
                };

                let _ = on_frame.send(VideoMessage::ControlResponse(
                    ControlResponse::RegisterState(state),
                ));
            }
            ControlRequest::PpuState => {
                let ppu = &nes.cpu.mem.ppu;
                let state = PpuState {
                    cycle: ppu.cycle(),
                    frame: ppu.frame(),
                    scanline: ppu.scanline(),
                    pixel: ppu.pixel(),

                    ctrl: ppu.ppuctrl.clone(),
                    mask: ppu.ppumask.clone(),
                    status: ppu.ppustatus.clone(),
                    addr: 0,
                    data: 0,
                    oamaddr: ppu.oam.addr(),
                    oamdata: ppu.oam.peekb(0x2004),
                    oamdma: 0,
                    scroll_x: ppu.scroll_x(),
                    scroll_y: ppu.scroll_y(),
                };

                let res = ControlResponse::PpuState(state);
                let _ = on_frame.send(VideoMessage::ControlResponse(res));
            }
        },
    }
}

/// Run the emulator using the system's sound card to control emulation speed.
/// The emulation is initially paused. Send a ControlMessage to start it.
pub fn run(
    rom: Rom,
    on_frame: Sender<VideoMessage>,
    on_control: Receiver<ControlMessage>,
) -> Stream {
    let mut nes = Nes::with_rom(rom);
    let mut state = EmulationState::Pause;
    let input_state = Default::default();
    let mut frame_buffer = FrameBuffer::new();

    let signal = dasp::signal::gen_mut(move || {
        for message in on_control.try_iter() {
            handle_control_message(message, &mut nes, &mut state, &on_frame, &mut frame_buffer);
        }

        match state {
            EmulationState::Pause | EmulationState::Kill => 0.0f32,
            EmulationState::Run | EmulationState::Step => {
                let step_result = nes.step(&input_state);
                let sample = nes.cpu.mem.apu.sample();

                if state == EmulationState::Step {
                    state = EmulationState::Pause;
                }

                if step_result.new_frame {
                    let frame = mem::replace(
                        &mut nes.cpu.mem.ppu.screen,
                        frame_buffer
                            .get()
                            .expect("No frames available to send back to the client!"),
                    );
                    let message = VideoMessage::FrameAvailable(frame);

                    // Discard send errors - if the other end of the channel hung up
                    // we are most likely shutting down
                    let _ = on_frame.send(message);
                }

                sample
            }
        }
    });

    let linear = Linear::new(0.0f32, 0.0f32);
    let mut signal = signal.from_hz_to_hz(linear, CPU_FREQUENCY, AUDIO_SAMPLE_RATE as f64);

    let output_device = get_output_device().expect("Unable to find an audio playback device");
    println!("Using output device: {}", output_device.name().unwrap());
    let suitable_config = start_audio_stream(&output_device).expect("Unable to start audio stream");

    output_device
        .build_output_stream(
            &suitable_config,
            move |a: &mut [f32], _b| {
                let samples_to_collect = a.len();

                for i in 0..samples_to_collect {
                    let sample = signal.next();
                    a[i] = sample;
                }
            },
            |err| panic!("Error playing audio: {:?}", err),
            None,
        )
        .expect("Unable to build audio stream")
}

fn get_output_device() -> Option<Device> {
    let host = cpal::default_host();

    host.output_devices()
        .unwrap()
        .into_iter()
        .filter_map(|device| {
            let devname = device.name().ok()?;

            let b = device
                .supported_output_configs()
                .ok()?
                .any(|cfg| stream_config_supported(&cfg));

            if !devname.contains("microphone") && b {
                Some(device)
            } else {
                None
            }
        })
        .next()
}

fn start_audio_stream(device: &Device) -> Option<StreamConfig> {
    let mut supported_configs = device.supported_output_configs().ok()?;

    Some(
        supported_configs
            .find(stream_config_supported)?
            .with_sample_rate(SampleRate(AUDIO_SAMPLE_RATE))
            .config(),
    )
}

fn stream_config_supported(cfg: &SupportedStreamConfigRange) -> bool {
    cfg.channels() == 1
        && cfg.sample_format() == SampleFormat::U8
        && cfg
            .try_with_sample_rate(SampleRate(AUDIO_SAMPLE_RATE))
            .is_some()
}
