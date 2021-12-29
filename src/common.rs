use std::path::{Path, PathBuf};

pub fn in_cwd<P: AsRef<Path>>(file: P) -> PathBuf {
    PathBuf::from(std::env::current_dir().unwrap().join(file))
}

pub const PW_FOLDERS: &str = "Keysight/PathWave/SignalGenerator";
