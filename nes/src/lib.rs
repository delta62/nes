#[macro_use]
mod byte;

mod audio;
mod cpu;
mod cpubus;
mod emulation;
mod frame_buffer;
mod input;
mod mapper;
mod mem;
mod nes;
mod ppu;
mod ram;
mod rom;

pub use crate::nes::Nes;
pub use cpu::{Cpu, Flags};
pub use emulation::{
    run, ControlMessage, ControlRequest, ControlResponse, EmulationState, PpuState, RegisterState,
    VideoMessage,
};
pub use frame_buffer::Frame;
pub use input::{ButtonState, Buttons, InputState};
pub use mem::Mem;
pub use ppu::{Ppu, PpuControl, PpuMask, PpuStatus, Rgb};
pub use rom::{Rom, RomLoadError};
