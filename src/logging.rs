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

#[derive(Serialize, Deserialize, Clone, Debug, Display)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Sink {
    RotatingFile {
        level: Level,
        name: String,
        file_name: String,
        truncate: Bool,
        max_size: u32,
        max_files: u8,
    },
    Console {
        level: Level,
        name: String,
        is_color: Bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Bool {
    Boolean(bool),
    String(String),
}

impl Bool {
    pub fn is_true(&self) -> bool {
        match self {
            Bool::Boolean(b) => b.clone(),
            Bool::String(s) => s == "true",
        }
    }

    fn from(value: bool) -> Bool {
        Bool::Boolean(value)
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
        is_color: Bool::from(true),
    };
    let d = Sink::RotatingFile {
        level: Level::Debug,
        name: "file".to_string(),
        truncate: Bool::from(true),
        max_files: 2,
        max_size: 1234,
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
