// mod cpu;
// mod debug;
// mod log;
mod nes;
// mod nametable;
// mod oam;
// mod palette;
// mod pattern;
// mod ppu;
mod arecord;
mod record;
mod screenshot;
mod texture;
mod view;

// pub use cpu::CpuView;
// pub use debug::DebugView;
// pub use self::log::LogView;
// pub use nametable::NametableView;
// pub use oam::OamView;
// pub use palette::PaletteView;
// pub use pattern::PatternView;
// pub use ppu::PpuView;
pub use arecord::AudioRecordView;
pub use record::RecordView;
pub use self::nes::NesView;
pub use view::View;
pub use screenshot::ScreenshotView;
