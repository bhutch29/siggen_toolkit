use crate::cli;
use crate::common::*;
use std::path::{Path, PathBuf};

fn serialize_channel(channel: &cli::SimulatedChannel) -> String {
    match channel {
        cli::SimulatedChannel::MCS15 => "hwPlatform=MCS15".to_string(),
        cli::SimulatedChannel::MCS31 { signal_count } => {
            format!("hwPlatform=MCS3;signalCount={}", signal_count)
        }
    }
}

fn serialize_channels(channels: Vec<cli::SimulatedChannel>) -> String {
    channels
        .iter()
        .map(|channel| format!("simulated {}\n", serialize_channel(channel)))
        .collect()
}

pub fn serialize_hwconfig(config: cli::SimulatedChannel, channel_count: u8) -> String {
    serialize_channels(vec![config; channel_count as usize])
}

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
    .into_iter()
    .flatten()
    .map(|x| {
        x.join("Keysight")
            .join("PathWave")
            .join("SignalGenerator")
            .join(FILE_NAME)
    })
    .collect()
}

pub fn set(path: &Path, config: cli::SimulatedChannel, channel_count: u8) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| std::fs::write(path, &serialize_hwconfig(config, channel_count)))?;
    Ok(())
}

pub fn read_from(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

pub const FILE_NAME: &str = "sghal_dev.cfg";
