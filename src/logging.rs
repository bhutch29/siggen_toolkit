use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use strum::{Display, EnumIter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoggingConfiguration {
    pub sinks: Vec<Sink>,
    pub loggers: Vec<Logger>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        file_name: String, // TODO: file_name can be optional
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
        Some(Bool::Boolean(bool)) => bool.clone(),
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

pub fn show() -> Result<()> {
    let a = Logger {
        name: "logger1".to_string(),
        level: Level::Trace,
        sinks: vec!["first".to_string(), "second".to_string()],
    };
    let b = Sink::Console {
        level: Level::Warn,
        name: "something".to_string(),
        is_color: Some(Bool::Boolean(true)),
    };
    let d = Sink::RotatingFile {
        level: Level::Debug,
        name: "file".to_string(),
        truncate: Some(Bool::Boolean(true)),
        max_files: Some(2),
        max_size: Some(1234),
        file_name: "temp.txt".to_string(),
    };
    let c = LoggingConfiguration {
        sinks: vec![b, d],
        loggers: vec![a],
    };
    let j = serde_json::to_string_pretty(&c)?;
    println!("{}", j);

    let text = fs::read_to_string("../siggen/static/ksflogger.cfg")?;
    let conf: LoggingConfiguration = serde_json::from_str(&text)?;
    println!("{:?}", conf);
    Ok(())
}
