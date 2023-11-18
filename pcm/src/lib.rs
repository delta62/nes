#[cfg_attr(target_os = "linux", path = "alsa.rs")]
#[cfg_attr(target_os = "macos", path = "mac.rs")]
mod device;

pub use self::device::*;
