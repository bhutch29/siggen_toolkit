use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use structopt::StructOpt;

// SigGen Toolkit supports navigation of SigGen deployments

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
struct Sgt {
    #[structopt(subcommand)]
    cmd: Option<Command>
}

#[derive(StructOpt, Debug)]
enum Command {
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
enum LogCommand {
    Set {
        config: String,
    }
}

#[derive(StructOpt, Debug)]
enum ReportCommand {
    Set {
        config: String,
    }
}

#[derive(StructOpt, Debug)]
enum SigGenCommand {
    Download {
        version: String,
    }
}

#[derive(StructOpt, Debug)]
enum HwConfigCommand {
    Set {
        #[structopt(default_value = "2", short, long)]
        channel_count: u8,
        #[structopt(subcommand)]
        config: SimulatedChannel,
    },
    Restore
}

#[derive(StructOpt, Debug)]
enum SimulatedChannel {
    MCS15,
    #[structopt(name = "MCS3")]
    MCS31 {
        #[structopt(default_value = "8", short, long)]
        signal_count: u8
    }
}

fn serialize_channel(channel: &SimulatedChannel) -> String {
    match channel {
        SimulatedChannel::MCS15 => "hwPlatform=MCS15".to_string(),
        SimulatedChannel::MCS31 { signal_count } => {
            format!("hwPlatform=MCS3;signalCount={}", signal_count)
        }
    }
}

fn serialize_channels(channels: &[SimulatedChannel]) -> String {
    channels.iter().map(|channel| format!("simulated {}\n", serialize_channel(channel))).collect()
}

fn try_read_file(path: &Path) -> Option<String> {
    path.exists()
        .then(|| fs::read_to_string(path).expect("Unable to read temp.txt"))
}

fn main() {
    let args: Sgt = Sgt::from_args();
    dbg!(&args);
    if args.cmd.is_none() {
        println!("no command");
        return
    }
    match args.cmd.unwrap() {
        Command::HwConfig(cmd) => {
            match cmd {
                HwConfigCommand::Set { config, channel_count } => {}
            }
        }
        Command::Log(cmd) => {
            match cmd {
                LogCommand::Set { config } => {}
            }
        }
        Command::SigGen(cmd) => {
            match cmd {
                SigGenCommand::Download { version } => {}
            }
        }
        Command::Report(cmd) => {
            match cmd {
                ReportCommand::Set { config } => {}
            }
        }
    }

    let temp_path = Path::new("/home/bhutch/projects/SigGenToolkit/temp.txt");
    // open::that(temp_path).expect("Unable to open temp.txt in default editor");

    // for entry in WalkDir::new("/home/bhutch/projects/SigGenToolkit") {
    //     let entry = entry.unwrap();
    //     println!("{}", entry.path().display());
    // }

    let contents = try_read_file(temp_path);
    if let Some(text) = &contents {
        println!("Contents before overwriting:");
        println!("{}", text);
    }

    let after = serialize_channels(&[
        SimulatedChannel::MCS15,
        SimulatedChannel::MCS31 { signal_count: 8 },
    ]);

    let write_file = || fs::write(&temp_path, &after).expect("Unable to write to temp.txt");

    match contents {
        None => {
            println!("Creating temp.txt");
            write_file();
        }
        Some(before) => {
            if before != after {
                println!("Overwriting temp.txt...");
                write_file();
            } else {
                println!("File already contains desired content");
            }
        }
    }
}
