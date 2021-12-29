use crate::cli;
use crate::common::*;
use anyhow::Result;
use std::fs;
use std::path::{PathBuf, Path};

fn serialize_channel(channel: &cli::SimulatedChannel) -> String {
    match channel {
        cli::SimulatedChannel::MCS15 => "hwPlatform=MCS15".to_string(),
        cli::SimulatedChannel::MCS31 { signal_count } => {
            format!("hwPlatform=MCS3;signalCount={}", signal_count)
        }
    }
}

fn serialize_channels(channels: &[cli::SimulatedChannel]) -> String {
    channels
        .iter()
        .map(|channel| format!("simulated {}\n", serialize_channel(channel)))
        .collect()
}

pub fn serialize_hwconfig(config: cli::SimulatedChannel, channel_count: u8) -> String {
    serialize_channels(vec![config; channel_count as usize].as_slice())
}

pub fn get_path() -> PathBuf {
    for path in valid_paths() {
        if path.exists() {
            return path;
        }
    }
    in_cwd(file_name())
}

pub fn valid_paths() -> Vec<PathBuf> {
    // TODO: SigGen first checks the cwd
    if cfg!(windows) {
        vec![dirs::document_dir(), Some(PathBuf::from("E:"))]
    } else {
        vec![dirs::home_dir()]
    }
    .into_iter()
    .filter_map(|x| x)
    .map(|x| {
        x.join("Keysight/PathWave/SignalGenerator")
            .join(file_name())
    })
    .collect()
}

pub fn set(path: &Path, config: cli::SimulatedChannel, channel_count: u8) -> Result<()> {
    if fs::create_dir_all(path.parent().unwrap()).is_ok() {
        fs::write(path, &serialize_hwconfig(config, channel_count))?;
    }
    Ok(())
}

// for entry in WalkDir::new("/home/bhutch/projects/siggen_toolkit") {
//     let entry = entry.unwrap();
//     println!("{}", entry.path().display());
// }

pub fn read_from(path: &Path) -> Option<String> {
    path.exists()
        .then(|| fs::read_to_string(path).ok())
        .flatten()
}

pub fn file_name() -> String {
    "sghal_dev.cfg".to_string()
}
