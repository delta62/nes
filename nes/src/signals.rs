use crate::{
    cpu::Flags,
    emulation::EmulationState,
    frame_buffer::Frame,
    input::Buttons,
    ppu::{PpuControl, PpuMask, PpuStatus},
    Rgb,
};

#[derive(Debug)]
pub enum ControlRequest {
    PatternTableContents,
    PpuState,
    RegisterState,
    PaletteState,
    RomContents,
}

#[derive(Debug)]
pub enum ControlMessage {
    ControllerInput {
        gamepad1: Buttons,
        gamepad2: Buttons,
    },
    EnableLogging(bool),
    SetCpuCycles(u64),
    SetState(EmulationState),
    SetProgramCounter(u16),
    RecycleFrame(Frame),
    ControlRequest(ControlRequest),
}

#[derive(Default, Debug, Clone)]
pub struct PaletteState {
    pub sprites: Vec<Rgb>,
    pub background: Vec<Rgb>,
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

#[derive(Debug, Clone)]
pub struct PatternTableContents {
    pub table1: Vec<u8>,
    pub table2: Vec<u8>,
}

#[derive(Debug)]
pub enum ControlResponse {
    PaletteState(PaletteState),
    PatternTableContents(PatternTableContents),
    PpuState(PpuState),
    RegisterState(RegisterState),
    RomContents(Vec<u8>),
}
