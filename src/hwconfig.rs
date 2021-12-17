use std::fs;
use std::path::{Path, PathBuf};
// use walkdir::WalkDir;
use crate::cli;
use std::borrow::Borrow;
use crate::common::*;

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

fn try_read_file(path: &Path) -> Option<String> {
    path.exists()
        .then(|| fs::read_to_string(path).ok())
        .flatten()
}

fn generate_hwconfig(config: cli::SimulatedChannel, channel_count: u8) -> String {
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
        .map(|x| x.join("Keysight/PathWave/SignalGenerator").join(file_name()))
        .collect()
}

pub fn set(config: cli::SimulatedChannel, channel_count: u8) {
    let text = generate_hwconfig(config, channel_count);

    let contents = try_read_file(get_path().borrow());
    if let Some(text) = &contents {
        println!("Contents before overwriting:");
        println!("{}", text);
    };

    let write_file = || {
        if fs::create_dir_all(get_path().parent().unwrap()).is_ok() {
            fs::write(get_path(), &text).expect("Unable to write to temp.txt");
        }
    };

    match contents {
        None => {
            println!("Creating temp.txt");
            write_file();
        }
        Some(before) => {
            if before != text {
                println!("Overwriting temp.txt...");
                write_file();
            } else {
                println!("File already contains desired content");
            }
        }
    }
}

// for entry in WalkDir::new("/home/bhutch/projects/siggen_toolkit") {
//     let entry = entry.unwrap();
//     println!("{}", entry.path().display());
// }

pub fn read() -> Option<String> {
    try_read_file(get_path().borrow())
}

pub fn file_name() -> String {
    "sghal_dev.cfg".to_string()
}
