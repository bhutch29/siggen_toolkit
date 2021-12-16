mod cli;
mod gui;
mod hwconfig;
mod logging;

use anyhow::Result;
use cli::*;
use std::fs::File;

fn main() -> Result<()> {
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
            HwConfigCommand::Path => println!("{}", hwconfig::get_path().to_string_lossy()),
        },
        Command::Log(cmd) => match cmd {
            LogCommand::Show { .. } => {
                logging::show()?;
            }
            LogCommand::Path => {println!("{}", logging::get_path().to_string_lossy())}
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
