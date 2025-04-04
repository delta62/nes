use crate::{
    frame_buffer::{Frame, FrameBuffer},
    log::log,
    mem::Mem,
    signals::{
        ControlMessage, ControlRequest, ControlResponse, PaletteState, PatternTableContents,
        PpuState, RegisterState,
    },
    InputState, Nes, Rgb, Rom,
};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, SampleFormat, SampleRate, Stream, StreamConfig, SupportedStreamConfigRange,
};
use dasp::{interpolate::linear::Linear, Signal};
use log::info;
use std::{
    mem,
    sync::mpsc::{Receiver, Sender},
};

const AUDIO_SAMPLE_RATE: u32 = 44_100;
const CPU_FREQUENCY: f64 = 1_789_773.0;

#[derive(Debug)]
pub enum VideoMessage {
    ControlResponse(ControlResponse),
    CpuStep(String),
    FrameAvailable(Frame),
    StateChanged(EmulationState),
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
    Run(Option<usize>),
}

fn handle_control_message(
    message: ControlMessage,
    nes: &mut Nes,
    state: &mut EmulationState,
    on_frame: &Sender<VideoMessage>,
    frame_buffer: &mut FrameBuffer,
    logging_enabled: &mut bool,
    step_limit: &mut Option<usize>,
) {
    match message {
        ControlMessage::SetState(s) => {
            if s != *state {
                *state = s;

                if let EmulationState::Run(limit) = s {
                    *step_limit = limit;
                }

                let msg = VideoMessage::StateChanged(s);
                let _ = on_frame.send(msg);
            }
        }
        ControlMessage::EnableLogging(enabled) => {
            *logging_enabled = enabled;
        }
        ControlMessage::ControllerInput { gamepad1, gamepad2 } => {
            nes.cpu.mem.input.set(&InputState {
                gamepad1: Some(gamepad1),
                gamepad2: Some(gamepad2),
            });
        }
        ControlMessage::RecycleFrame(frame) => frame_buffer.put(frame),
        ControlMessage::SetProgramCounter(pc) => nes.cpu.pc = pc,
        ControlMessage::SetCpuCycles(cy) => {
            nes.cpu.cy = cy;
            let ppu = &mut nes.cpu.mem.ppu;

            ppu.set_cycle_from_cpu(cy);
        }
        ControlMessage::ControlRequest(req) => match req {
            ControlRequest::RomContents => {
                let rom = nes.cpu.mem.mapper.as_ref();
                let buf = (0x4020..=0xFFFF).map(|addr| rom.peekb(addr)).collect();

                let res = ControlResponse::RomContents(buf);
                let _ = on_frame.send(VideoMessage::ControlResponse(res));
            }
            ControlRequest::PaletteState => {
                let mut background = Vec::with_capacity(16);
                let mut sprites = Vec::with_capacity(16);
                let vram = &nes.cpu.mem.ppu.vram;

                for x in 0..16 {
                    let addr = 0x3F00 + x;
                    let val = vram.peekb(addr);
                    let rgb = *Rgb::from_byte(val);
                    background.push(rgb);
                }

                for x in 0..16 {
                    let addr = 0x3F10 + x;
                    let val = vram.peekb(addr);
                    let rgb = *Rgb::from_byte(val);
                    sprites.push(rgb);
                }

                let state = PaletteState {
                    sprites,
                    background,
                };

                let res = ControlResponse::PaletteState(state);
                let _ = on_frame.send(VideoMessage::ControlResponse(res));
            }
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

                let res = ControlResponse::RegisterState(state);
                let _ = on_frame.send(VideoMessage::ControlResponse(res));
            }
            ControlRequest::PatternTableContents => {
                let vram = &nes.cpu.mem.ppu.vram;

                let mut table1 = vec![0; 128 * 128 * 3];
                let mut table2 = vec![0; 128 * 128 * 3];

                for i in 0..256 {
                    for row in 0..8 {
                        let addr = i * 16 + row;
                        let pattern_lo = vram.peekb(addr);
                        let pattern_hi = vram.peekb(addr + 8);

                        for col in 0..8 {
                            let lo = bitn!(pattern_lo, 7 - col);
                            let hi = bitn!(pattern_hi, 7 - col) << 1;
                            let pattern = lo + hi;

                            let base_tile_offset = i / 16 * 384 * 8 + i % 16 * 24;
                            let row_addr = base_tile_offset + row * 384;
                            let addr = (row_addr + col * 3) as usize;
                            let color = pattern * 75;

                            table1[addr] = color;
                            table1[addr + 1] = color;
                            table1[addr + 2] = color;
                        }
                    }
                }

                for i in 0..256 {
                    for row in 0..8 {
                        let addr = 1000 + i * 16 + row;
                        let pattern_lo = vram.peekb(addr);
                        let pattern_hi = vram.peekb(addr + 8);

                        for col in 0..8 {
                            let lo = bitn!(pattern_lo, 7 - col);
                            let hi = bitn!(pattern_hi, 7 - col) << 1;
                            let pattern = lo + hi;

                            let base_tile_offset = i / 16 * 384 * 8 + i % 16 * 24;
                            let row_addr = base_tile_offset + row * 384;
                            let addr = (row_addr + col * 3) as usize;
                            let color = pattern * 75;

                            table2[addr] = color;
                            table2[addr + 1] = color;
                            table2[addr + 2] = color;
                        }
                    }
                }

                let contents = PatternTableContents { table1, table2 };
                let msg = ControlResponse::PatternTableContents(contents);
                let _ = on_frame.send(VideoMessage::ControlResponse(msg));
            }
            ControlRequest::PpuState => {
                let ppu = &nes.cpu.mem.ppu;
                let state = PpuState {
                    cycle: ppu.cycle(),
                    frame: ppu.frame(),
                    scanline: ppu.scanline(),
                    pixel: ppu.pixel(),

                    ctrl: ppu.ppuctrl,
                    mask: ppu.ppumask,
                    status: ppu.ppustatus,
                    addr: 0,
                    data: 0,
                    oamaddr: ppu.oam.addr(),
                    oamdata: 0,
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
    let mut frame_buffer = FrameBuffer::new();
    let mut logging_enabled = false;
    let mut step_limit = None;

    let signal = dasp::signal::gen_mut(move || {
        for message in on_control.try_iter() {
            handle_control_message(
                message,
                &mut nes,
                &mut state,
                &on_frame,
                &mut frame_buffer,
                &mut logging_enabled,
                &mut step_limit,
            );
        }

        match state {
            EmulationState::Pause | EmulationState::Kill => 0.0f32,
            EmulationState::Run(_) | EmulationState::Step => {
                let step_result = nes.step();
                let sample = nes.cpu.mem.apu.sample();

                if logging_enabled && !nes.cpu.is_busy() {
                    let log_str = log(&nes);
                    let _ = on_frame.send(VideoMessage::CpuStep(log_str));
                }

                if state == EmulationState::Step && !nes.cpu.is_busy() {
                    state = EmulationState::Pause;
                    let _ = on_frame.send(VideoMessage::StateChanged(state));
                }

                if let EmulationState::Run(_) = state {
                    if !nes.cpu.is_busy() {
                        if let Some(n) = step_limit.as_mut() {
                            if *n <= 1 {
                                state = EmulationState::Pause;
                                let _ = on_frame.send(VideoMessage::StateChanged(state));
                            } else {
                                *n -= 1;
                            }
                        }
                    }
                }

                if step_result.new_frame {
                    let frame = mem::replace(&mut nes.cpu.mem.ppu.screen, frame_buffer.get());
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
    info!(
        "Using output device {}",
        output_device
            .name()
            .unwrap_or_else(|_| "UNKNOWN".to_owned())
    );
    let suitable_config = start_audio_stream(&output_device).expect("Unable to start audio stream");

    output_device
        .build_output_stream(
            &suitable_config,
            move |a: &mut [f32], _b| {
                let samples_to_collect = a.len();

                for frame in a.iter_mut().take(samples_to_collect) {
                    let sample = signal.next();
                    *frame = sample;
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
