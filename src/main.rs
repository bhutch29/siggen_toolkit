mod cli;
mod hwconfig;

use cli::*;
use std::fs::File;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: move all of this into cli.rs
    let args: Sgt = Sgt::from_args();
    dbg!(&args);
    if args.cmd.is_none() {
        // TODO: launch GUI
        println!("no command");
        return Ok(());
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
            LogCommand::Show { .. } => {}
            LogCommand::Path => {}
            LogCommand::Open => {}
        },
        Command::SigGen(cmd) => match cmd {
            SigGenCommand::Download { version: _ } => {
                // TODO
                let response = reqwest::blocking::get("https://artifactory.it.keysight.com/artifactory/generic-local-pwsg/siggen/packages-linux/develop/siggen_1-9-1-9_2021-11-22_linux.zip")?;
                let bytes = response.bytes()?;
                let mut out = File::create("/home/bhutch/projects/SigGenToolkit/temp.zip").expect("failed to create file");
                std::io::copy(&mut bytes.as_ref(), &mut out).expect("failed to copy content");
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
