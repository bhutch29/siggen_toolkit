pub use structopt::StructOpt;
use strum::Display;

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
    Show {},
    Paths,
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
