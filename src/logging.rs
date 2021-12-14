use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoggingConfiguration {
    pub sinks: Vec<Sink>,
    pub loggers: Vec<Logger>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Logger {
    pub name: String,
    pub level: Level,
    pub sinks: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Sink {
    RotatingFile {
        level: Level,
        name: String,
        file_name: String,
        truncate: bool,
        max_size: u32,
        max_files: u8,
    },
    Console {
        level: Level,
        name: String,
        is_color: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}
