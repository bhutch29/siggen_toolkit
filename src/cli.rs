use std::fmt;
pub use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "sgt",
    about = "Navigate SigGen deployments.\n\nRun without arguments to launch GUI."
)]
pub struct Sgt {
    #[structopt(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(name = "hwconfig", about = "Simulated hardware configuration.")]
    HwConfig(HwConfigCommand),
    #[structopt(about = "Logging configuration.")]
    Log(LogCommand),
    #[structopt(name = "siggen", about = "Download or run application versions.")]
    SigGen(SigGenCommand),
    #[structopt(about = "Create or browse reports.")]
    Report(ReportCommand),
    #[structopt(about = "Windows Event Log viewing.")]
    Events(EventsCommand),
}

#[derive(StructOpt, Debug)]
pub enum LogCommand {
    Set {},
    Show {},
    Path,
}

#[derive(StructOpt, Debug)]
pub enum ReportCommand {
    Zip {},
    Upload {
        // need to be separate or just have it upload as part of Zip?
    },
    List {},
    Download {},
}

#[derive(StructOpt, Debug)]
pub enum SigGenCommand {
    Download { version: String },
    List,
    Run {},
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
    Path,
}

#[derive(StructOpt, Debug, Clone, Copy, PartialEq)]
pub enum SimulatedChannel {
    MCS15,
    #[structopt(name = "MCS3")]
    MCS31 {
        #[structopt(default_value = "8", short, long)]
        signal_count: u8,
    },
}

impl fmt::Display for SimulatedChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SimulatedChannel::MCS15 => {
                write!(f, "MCS1.5")
            }
            SimulatedChannel::MCS31 { .. } => {
                write!(f, "MCS3.1")
            }
        }
    }
}
