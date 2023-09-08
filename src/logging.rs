use crate::common::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;
use strum::{Display, EnumIter, EnumString};

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
    Err,
    Critical,
    Off,
}

impl Default for Level {
    fn default() -> Self {
        Level::Off
    }
}

#[derive(Debug, Copy, Clone, EnumIter, EnumString, Display)]
pub enum Template {
    #[strum(serialize = "General Purpose")]
    GeneralPurpose,
    #[strum(serialize = "Monitor Sghal Setups")]
    MonitorSghalSetups,
    Mobius,
    Websockets,
    #[strum(serialize = "MultiInstrument")]
    MultiInstrumentGrpc,
    Licensing,
}

pub fn get_config_path() -> Option<PathBuf> {
    for path in valid_paths() {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn get_config_path_or_cwd() -> PathBuf {
    get_config_path().unwrap_or_else(|| in_cwd(FILE_NAME))
}

pub fn valid_paths() -> Vec<PathBuf> {
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

pub fn get_config_from(path: &Path) -> Option<LoggingConfiguration> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

pub fn set_config(path: &Path, config: LoggingConfiguration) -> anyhow::Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

pub fn get_code_defined_log_path() -> PathBuf {
    PathBuf::from(CODE_DEFINED_LOG_PATH)
}

pub fn get_log_path_from_current_config() -> PathBuf {
    get_config_from(&get_config_path_or_cwd())
        .and_then(|config| {
            config.sinks.iter().find_map(|sink| match sink {
                Sink::File { file_name, .. }
                | Sink::DailyFile { file_name, .. }
                | Sink::RotatingFile { file_name, .. } => Some(PathBuf::from(file_name)),
                _ => None,
            })
        })
        .unwrap_or_else(|| get_code_defined_log_path())
}

pub fn get_exception_log_path() -> PathBuf {
    PathBuf::from(EXCEPTION_LOG_PATH)
}

pub fn show() -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&get_config_from(&get_config_path_or_cwd()))?
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

#[cfg(windows)]
const CODE_DEFINED_LOG_PATH: &str = r"C:\Temp\Keysight.PathWave.SG.log";

// TODO: ARM path
#[cfg(not(windows))]
const CODE_DEFINED_LOG_PATH: &str = "/tmp/Keysight.PathWave.SG.log";

#[cfg(windows)]
const EXCEPTION_LOG_PATH: &str = r"C:\Temp\Keysight.PathWave.SG.ExceptionLog.txt";

// TODO: ARM path
#[cfg(not(windows))]
const EXCEPTION_LOG_PATH: &str = "/tmp/Keysight.PathWave.SG.ExceptionLog.txt";

lazy_static! {
    static ref DEFAULT_SINKS: Vec<Sink> = vec![
        Sink::Console {
            level: Level::Trace,
            name: "console".to_string(),
            is_color: Some(Bool::Boolean(false)),
        },
        Sink::RotatingFile {
            level: Level::Trace,
            name: "file".to_string(),
            file_name: get_code_defined_log_path().to_string_lossy().to_string(),
            truncate: None,
            max_size: Some(1048576),
            max_files: Some(5),
        }
    ];
    static ref TEMPLATE_GENERAL_PURPOSE: LoggingConfiguration = LoggingConfiguration {
        sinks: DEFAULT_SINKS.clone(),
        loggers: vec![
            Logger {
                name: "*".to_string(),
                level: Level::Warn,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen".to_string(),
                level: Level::Info,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.*".to_string(),
                level: Level::Info,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
        ]
    };
    static ref TEMPLATE_SGHAL_SETUPS: LoggingConfiguration = LoggingConfiguration {
        sinks: DEFAULT_SINKS.clone(),
        loggers: vec![
            Logger {
                name: "*".to_string(),
                level: Level::Warn,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.sghal".to_string(),
                level: Level::Debug,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
        ]
    };

    static ref TEMPLATE_MOBIUS: LoggingConfiguration = LoggingConfiguration {
        sinks: DEFAULT_SINKS.clone(),
        loggers: vec![
            Logger {
                name: "*".to_string(),
                level: Level::Warn,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.mobius".to_string(),
                level: Level::Debug,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
        ]
    };

    static ref TEMPLATE_WEBSOCKETS: LoggingConfiguration = LoggingConfiguration {
        sinks: DEFAULT_SINKS.clone(),
        loggers: vec![
            Logger {
                name: "*".to_string(),
                level: Level::Warn,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.websocket".to_string(),
                level: Level::Debug,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.iws".to_string(),
                level: Level::Debug,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
        ]
    };

    static ref TEMPLATE_MULTI_INSTRUMENT: LoggingConfiguration = LoggingConfiguration {
        sinks: DEFAULT_SINKS.clone(),
        loggers: vec![
            Logger {
                name: "*".to_string(),
                level: Level::Warn,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.grpc".to_string(),
                level: Level::Debug,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
        ]
    };

    static ref TEMPLATE_LICENSING: LoggingConfiguration = LoggingConfiguration {
        sinks: DEFAULT_SINKS.clone(),
        loggers: vec![
            Logger {
                name: "*".to_string(),
                level: Level::Warn,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
            Logger {
                name: "siggen.licensing".to_string(),
                level: Level::Debug,
                sinks: vec!["console".to_string(), "file".to_string()],
            },
        ]
    };
}

pub fn get_template(template: &Template) -> LoggingConfiguration {
    match template {
        Template::GeneralPurpose => {TEMPLATE_GENERAL_PURPOSE.clone()}
        Template::MonitorSghalSetups => {TEMPLATE_SGHAL_SETUPS.clone()}
        Template::Mobius => {TEMPLATE_MOBIUS.clone()}
        Template::Websockets => {TEMPLATE_WEBSOCKETS.clone()}
        Template::MultiInstrumentGrpc => {TEMPLATE_MULTI_INSTRUMENT.clone()}
        Template::Licensing => {TEMPLATE_LICENSING.clone()}
    }
}
