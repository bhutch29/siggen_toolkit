mod cli;
mod hwconfig;

use cli::*;

fn main() {
    // TODO: move all of this into cli.rs
    let args: Sgt = Sgt::from_args();
    dbg!(&args);
    if args.cmd.is_none() {
        // TODO: launch GUI
        println!("no command");
        return;
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
            HwConfigCommand::Open => hwconfig::open(),
        },
        Command::Log(cmd) => match cmd {
            LogCommand::Set { .. } => {}
            LogCommand::Show { .. } => {}
            LogCommand::Path => {}
            LogCommand::Open => {}
        },
        Command::SigGen(cmd) => match cmd {
            SigGenCommand::Download { version: _ } => {}
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
    }
}
