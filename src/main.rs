mod cli;
mod common;
mod gui;
mod gui_state;
mod hwconfig;
mod logging;
mod versions;
mod events;

use crate::versions::VersionsClient;
use anyhow::Result;
use cli::*;

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
            } => hwconfig::set(&hwconfig::get_path(), config, channel_count)?,
            HwConfigCommand::Restore => println!("Not yet implemented!"),
            HwConfigCommand::Show => match hwconfig::read_from(&hwconfig::get_path()) {
                Some(text) => {
                    println!("{}", text)
                }
                None => {
                    println!("No hwconfig found")
                }
            },
            HwConfigCommand::Paths => {
                for path in hwconfig::valid_paths() {
                    println!("{} {}", path.display(), path.exists())
                }
            }
        },
        Command::Log(cmd) => match cmd {
            LogCommand::Show { .. } => {
                logging::show()?;
            }
            LogCommand::Paths => {
                for path in logging::valid_paths() {
                    println!("{} {}", path.display(), path.exists())
                }
            }
        },
        Command::SigGen(cmd) => match cmd {
            SigGenCommand::Download { version: _ } => {
                VersionsClient::default().do_stuff()?;
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
            EventsCommand::List { .. } => {
                if cfg!(windows) {
                    events::event_stuff();
                }
            }
        },
    };
    Ok(())
}
