mod cli;
mod gui;
mod hwconfig;
mod logging;

use cli::*;
use std::error::Error;
use std::fs;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Sgt = Sgt::from_args();
    dbg!(&args);
    if args.cmd.is_none() {
        gui::run();
    }

    match args.cmd.unwrap() {
        Command::HwConfig(cmd) => match cmd {
            HwConfigCommand::Set {
                config,
                channel_count,
            } => hwconfig::set(config, channel_count),
            HwConfigCommand::Restore => println!("Not yet implemented!"),
            HwConfigCommand::Show => match hwconfig::read() {
                Some(text) => {
                    println!("{}", text)
                }
                None => {
                    println!("No hwconfig found")
                }
            },
            HwConfigCommand::Path => println!("{}", hwconfig::get_path().to_str().unwrap()),
        },
        Command::Log(cmd) => match cmd {
            LogCommand::Set { .. } => {}
            LogCommand::Show { .. } => {
                let a = logging::Logger {
                    name: "logger1".to_string(),
                    level: logging::Level::Trace,
                    sinks: vec!["first".to_string(), "second".to_string()],
                };
                let b = logging::Sink::Console {
                    level: logging::Level::Warn,
                    name: "something".to_string(),
                    is_color: true,
                };
                let d = logging::Sink::RotatingFile {
                    level: logging::Level::Debug,
                    name: "file".to_string(),
                    truncate: true,
                    max_files: 2,
                    max_size: 1234,
                    file_name: "temp.txt".to_string(),
                };
                let c = logging::LoggingConfiguration {
                    sinks: vec![b, d],
                    loggers: vec![a],
                };
                let j = serde_json::to_string_pretty(&c)?;
                println!("{}", j);

                let text = fs::read_to_string("../siggen/static/ksflogger.cfg")?;
                let conf: logging::LoggingConfiguration = serde_json::from_str(&text)?;
                println!("{:?}", conf);
            }
            LogCommand::Path => {}
        },
        Command::SigGen(cmd) => match cmd {
            SigGenCommand::Download { version: _ } => {
                // TODO
                let response = reqwest::blocking::get("https://artifactory.it.keysight.com/artifactory/generic-local-pwsg/siggen/packages-linux/develop/siggen_1-9-1-9_2021-11-22_linux.zip")?;
                let bytes = response.bytes()?;
                let mut out = File::create("/home/bhutch/projects/SigGenToolkit/temp.zip")?;
                std::io::copy(&mut bytes.as_ref(), &mut out)?;
            }
            SigGenCommand::List => {}
            SigGenCommand::Run { .. } => {}
        },
        Command::Report(cmd) => match cmd {
            ReportCommand::Download { .. } => {}
            ReportCommand::Zip { .. } => {}
            ReportCommand::Upload { .. } => {}
            ReportCommand::List { .. } => {}
        },
        Command::Events(cmd) => match cmd {
            EventsCommand::List { .. } => {}
        },
    };
    Ok(())
}
