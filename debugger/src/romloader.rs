use nes::{Rom, RomLoadError};
use std::path::Path;
use std::fs::File;

pub struct RomLoader;

impl RomLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Rom, RomLoadError> {
        let mut file = File::open(path)?;
        Rom::from_path(&mut file)
    }
}
