/// TODO
/// This module is only here so that the project can compile on OSX. It does not work
/// at all.

use std::collections::VecDeque;

#[derive(Debug)]
pub struct Device;

#[derive(Debug)]
pub enum Error { }

pub struct DeviceConfig {
    /// The target amount of time to store buffered audio for. The sound driver will use something
    /// close to this number, but it might not be exact.
    pub buffer_target_us: u32,
    /// The number of channels for playback. Channel data is always interleaved.
    pub channels: u32,
    /// The target amount of time to process before asking the application for more data. The sound
    /// driver will use something close to this number, but it might not be exact.
    pub period_target_us: u32,
    /// The constant sample rate in hz to output audio at
    pub sample_rate: u32,
}

impl Device {
    pub fn with_config(_config: DeviceConfig) -> Result<Self, Error> {
        Ok(Self { })
    }

    pub fn run<F>(self, mut _data_callback: F)
    where F: FnMut(&mut VecDeque<f32>, usize) {
    }
}
