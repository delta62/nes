#[macro_use]
mod byte;

mod audio;
mod cpu;
mod cpubus;
mod emulation;
mod frame_buffer;
mod input;
mod log;
mod mapper;
mod mem;
mod nes;
mod ppu;
mod ram;
mod rom;
mod signals;

pub use crate::nes::Nes;
pub use cpu::{Cpu, Flags};
pub use emulation::{run, EmulationState, VideoMessage};
pub use frame_buffer::Frame;
pub use input::{Buttons, InputState};
pub use mem::Mem;
pub use ppu::{Ppu, PpuControl, PpuMask, PpuStatus, Rgb};
pub use rom::{INesHeader, Rom, RomLoadError};
pub use signals::{
    ControlMessage, ControlRequest, ControlResponse, PaletteState, PatternTableContents, PpuState,
    RegisterState,
};
