use crate::common::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use strum::{Display, EnumIter};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LoggingConfiguration {
    pub sinks: Vec<Sink>,
    pub loggers: Vec<Logger>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Logger {
    pub name: String,
    pub level: Level,
    pub sinks: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Display, EnumIter)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Sink {
    File {
        level: Level,
        name: String,
        file_name: String,
        truncate: Option<Bool>,
    },
    RotatingFile {
        level: Level,
        name: String,
        file_name: String,
        truncate: Option<Bool>,
        max_size: Option<u32>,
        max_files: Option<u8>,
    },
    DailyFile {
        level: Level,
        name: String,
        file_name: String,
        truncate: Option<Bool>,
    },
    Console {
        level: Level,
        name: String,
        is_color: Option<Bool>,
    },
    Etw {
        level: Level,
        name: String,
        activities_only: Option<Bool>,
    },
    Windiag {
        level: Level,
        name: String,
    },
    EventLog {
        level: Level,
        name: String,
    },
    Nats {
        level: Level,
        name: String,
        url: String,
    },
}

impl Sink {
    pub fn get_name(&self) -> &String {
        match self {
            Sink::RotatingFile { name, .. } => name,
            Sink::Console { name, .. } => name,
            Sink::File { name, .. } => name,
            Sink::DailyFile { name, .. } => name,
            Sink::Etw { name, .. } => name,
            Sink::Windiag { name, .. } => name,
            Sink::EventLog { name, .. } => name,
            Sink::Nats { name, .. } => name,
        }
    }

    pub fn get_name_and_level_as_mut(&mut self) -> (&mut String, &mut Level) {
        match self {
            Sink::RotatingFile {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::Console {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::File {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::DailyFile {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::Etw {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::Windiag {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::EventLog {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
            Sink::Nats {
                ref mut name,
                ref mut level,
                ..
            } => (name, level),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Bool {
    Boolean(bool),
    String(String),
}

pub fn is_true(value: &Option<Bool>) -> bool {
    match value {
        None => false,
        Some(Bool::Boolean(bool)) => *bool,
        Some(Bool::String(string)) => string == "true",
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone, EnumIter, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
    Off,
}

impl Default for Level {
    fn default() -> Self {
        Level::Off
    }
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
    if cfg!(windows) {
        vec![dirs::document_dir(), Some(PathBuf::from("E:"))]
    } else {
        vec![dirs::home_dir()]
    }
    .into_iter()
    .flatten()
    .map(|x| x.join(PW_FOLDERS).join(FILE_NAME))
    .collect()
}

pub fn get_config_from(path: &Path) -> Option<LoggingConfiguration> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

pub fn set_config(path: &Path, config: LoggingConfiguration) -> anyhow::Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

pub fn get_log_path() -> PathBuf {
    get_config_from(&get_path_or_cwd())
        .and_then(|config| {
            config.sinks.iter().find_map(|sink| match sink {
                Sink::File { file_name, .. }
                | Sink::DailyFile { file_name, .. }
                | Sink::RotatingFile { file_name, .. } => Some(PathBuf::from(file_name)),
                _ => None,
            })
        })
        .unwrap_or_else(|| PathBuf::from(CODE_DEFINED_LOG_PATH))
}

pub fn show() -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&get_config_from(&get_path_or_cwd()))?
    );
    Ok(())
}

pub fn remove_invalid_sinks(logger: &mut Logger, sinks: &[Sink]) {
    logger.sinks.retain(|logger_sink_name| {
        sinks
            .iter()
            .any(|target_sink| target_sink.get_name() == logger_sink_name)
    });
}

pub const FILE_NAME: &str = "ksflogger.cfg";

#[cfg(target_os = "windows")]
const CODE_DEFINED_LOG_PATH: &str = r"C:\Temp\Keysight.PathWave.SG.log";

#[cfg(not(target_os = "windows"))]
const CODE_DEFINED_LOG_PATH: &str = "/tmp/Keysight.PathWave.SG.log";
