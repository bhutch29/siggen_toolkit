use crate::events;
use crate::hwconfig;
use crate::logging;
use crate::report;
use crate::versions;
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
        name: String,
    },
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
        #[structopt(short, long)]
        has_io_extender: bool,
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
        #[structopt(short, long)]
        has_io_extender: bool,
    },
}

impl Default for SimulatedChannel {
    fn default() -> Self {
        Self::MCS31 {has_io_extender: false}
    }
}

pub fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::HwConfig(cmd) => match cmd {
            HwConfigCommand::Set { config, channel_count, has_io_extender } => {
                hwconfig::set(&hwconfig::get_path_or_cwd(), config, channel_count, has_io_extender)?
            }
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
            ReportCommand::Zip { name, force } => {
                let file_name = report::zip_file_name(&name);
                if !force && Path::new(&file_name).exists() {
                    return Err(anyhow::anyhow!(
                        "Destination file already exists: {}\n\
                         Consider using the --force flag or using a unique name.",
                        file_name
                    ));
                }
                println!("{}", file_name);
                report::create_report(&name)?;
            }
            ReportCommand::Upload { name } => {
                let file_name = report::zip_file_name(&name);
                let client = versions::VersionsClient::default();
                let handle = client.upload_report(Path::new(&file_name), None, None)?;
                if handle.join().is_err() {
                    return Err(anyhow::anyhow!("unknown error occurred when uploading"));
                }
                println!(
                    "{}/{}/{}",
                    versions::BASE_FILE_URL,
                    versions::report_segments(),
                    file_name
                );
            }
        },
        Command::Events(cmd) => match cmd {
            EventsCommand::List { .. } => {
                events::print_event_stuff();
            }
        },
    };
    Ok(())
}
