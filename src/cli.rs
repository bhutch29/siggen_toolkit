pub use structopt::StructOpt;

// sgt (open GUI)
// sgt hwconfig set
// sgt hwconfig restore
// sgt hwconfig show
// sgt hwconfig path
// sgt hwconfig open
// sgt log set
// sgt log show
// sgt log path
// sgt log open
// sgt events show
// sgt siggen download
// sgt siggen list
// sgt siggen run
// sgt report zip
// sgt report upload (or make it the default?)
// sgt report list
// sgt report download

// * Upload reports to Artifactory


#[derive(StructOpt, Debug)]
#[structopt(name = "sgt", about = "Navigate SigGen deployments.\n\nRun without arguments to launch GUI.")]
pub struct Sgt {
    #[structopt(subcommand)]
    pub cmd: Option<Command>
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
}

#[derive(StructOpt, Debug)]
pub enum LogCommand {
    Set {
        config: String,
    }
}

#[derive(StructOpt, Debug)]
pub enum ReportCommand {
    Set {
        config: String,
    }
}

#[derive(StructOpt, Debug)]
pub enum SigGenCommand {
    Download {
        version: String,
    }
}

#[derive(StructOpt, Debug)]
pub enum HwConfigCommand {
    Set {
        #[structopt(default_value = "2", short, long)]
        channel_count: u8,
        #[structopt(subcommand)]
        config: SimulatedChannel,
    },
    Restore
}

#[derive(StructOpt, Debug)]
pub enum SimulatedChannel {
    MCS15,
    #[structopt(name = "MCS3")]
    MCS31 {
        #[structopt(default_value = "8", short, long)]
        signal_count: u8
    }
}
