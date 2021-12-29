use std::path::{Path, PathBuf};
use std::process;

pub fn in_cwd<P: AsRef<Path>>(file: P) -> PathBuf {
    PathBuf::from(std::env::current_dir().unwrap().join(file))
}

pub const PW_FOLDERS: &str = "Keysight/PathWave/SignalGenerator";

pub fn open_explorer(path: &Path) -> anyhow::Result<()> {
    process::Command::new(if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    })
    .arg(path.parent().unwrap())
    .spawn()?;
    Ok(())
}
