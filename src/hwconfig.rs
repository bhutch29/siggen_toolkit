use crate::common::*;
use std::path::{Path, PathBuf};

pub fn get_path() -> Option<PathBuf> {
    for path in valid_paths() {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn get_path_or_cwd() -> PathBuf {
    get_path().unwrap_or_else(|| in_cwd(FILE_NAME))
}

pub fn valid_paths() -> Vec<PathBuf> {
    // TODO: SigGen first checks the cwd
    if cfg!(windows) {
        vec![dirs::document_dir(), Some(PathBuf::from("E:\\"))]
    } else {
        vec![dirs::home_dir()]
    }
    .iter()
    .flatten()
    .map(|x| {
        x.join("Keysight")
            .join("PathWave")
            .join("SignalGenerator")
            .join(FILE_NAME)
    })
    .collect()
}

pub fn set_text(path: &Path, text: &String) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| std::fs::write(path, text))?;
    Ok(())
}

pub fn read_from(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

pub const FILE_NAME: &str = "sghal_dev.cfg";
