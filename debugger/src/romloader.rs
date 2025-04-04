use log::info;
use nes::{Rom, RomLoadError};
use std::{fs::File, path::Path};

pub struct RomLoader;

impl RomLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Rom, RomLoadError> {
        let mut file = File::open(path.as_ref())?;
        let ret = Rom::from_path(&mut file)?;

        info!(
            "Loaded {:?}",
            path.as_ref()
                .file_stem()
                .and_then(|fs| fs.to_str())
                .unwrap_or("")
        );

        Ok(ret)
    }
}
