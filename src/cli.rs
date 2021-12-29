use crate::events;
use crate::hwconfig;
use crate::logging;
use crate::report;
use std::path::Path;
pub use structopt::StructOpt;
use strum::Display;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "sgt",
    about = "Navigate SigGen deployments.\n\nRun without arguments to launch GUI."
)]
pub struct Sgt {
    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(name = "hwconfig", about = "Simulated hardware configuration.")]
    HwConfig(HwConfigCommand),
    #[structopt(about = "Logging configuration.")]
    Log(LogCommand),
    #[structopt(about = "Create or browse reports.")]
    Report(ReportCommand),
    #[structopt(about = "Windows Event Log viewing.")]
    Events(EventsCommand),
}

#[derive(StructOpt, Debug)]
pub enum LogCommand {
    Show {},
    Paths,
    SinkPath,
}

#[derive(StructOpt, Debug)]
pub enum ReportCommand {
    Zip {
        name: String,
        #[structopt(short, long, about = "Overwrite file if necessary.")]
        force: bool,
    },
    Upload {
        // need to be separate or just have it upload as part of Zip?
    },
    List {},
    Download {},
}

#[derive(StructOpt, Debug)]
pub enum EventsCommand {
    List,
}

#[derive(StructOpt, Debug)]
pub enum HwConfigCommand {
    Set {
        #[structopt(default_value = "2", short, long)]
        channel_count: u8,
        #[structopt(subcommand)]
        config: SimulatedChannel,
    },
    Restore,
    Show,
    Paths,
}

#[derive(StructOpt, Debug, Clone, Copy, PartialEq, Display)]
pub enum SimulatedChannel {
    #[strum(serialize = "MCS1.5")]
    MCS15,
    #[strum(serialize = "MCS3.1")]
    #[structopt(name = "MCS3")]
    MCS31 {
        #[structopt(default_value = "8", short, long)]
        signal_count: u8,
    },
}

impl Default for SimulatedChannel {
    fn default() -> Self {
        Self::MCS31 { signal_count: 1 }
    }
}

pub fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::HwConfig(cmd) => match cmd {
            HwConfigCommand::Set {
                config,
                channel_count,
            } => hwconfig::set(&hwconfig::get_path_or_cwd(), config, channel_count)?,
            HwConfigCommand::Restore => println!("Not yet implemented!"),
            HwConfigCommand::Show => match hwconfig::read_from(&hwconfig::get_path_or_cwd()) {
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
            LogCommand::SinkPath => {
                println!("{}", logging::get_log_path().display());
            }
        },
        Command::Report(cmd) => match cmd {
            ReportCommand::Download { .. } => {}
            ReportCommand::Zip { name, force } => {
                let file_name = report::zip_file_name(&name);
                if !force && Path::new(&file_name).exists() {
                    return Err(anyhow::anyhow!(
                        "Destination file already exists: {}\n\
                         Consider using the --force flag or using a unique name.",
                        file_name
                    ));
                }
                println!("File Name: {}", file_name);
                report::create_report(&name)?;
            }
            ReportCommand::Upload { .. } => {}
            ReportCommand::List { .. } => {}
        },
        Command::Events(cmd) => match cmd {
            EventsCommand::List { .. } => {
                events::event_stuff();
            }
        },
    };
    Ok(())
}
