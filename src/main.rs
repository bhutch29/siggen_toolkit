// TODO: This works to hide the console when launching the GUI but then hides the console output when using the CLI
// #![windows_subsystem = "windows"] // Hides console on Windows

use structopt::StructOpt;
use crate::cli::{Command, Sgt};

mod cli;
mod common;
mod gui;
mod gui_state;
mod hwconfig;
mod logging;
mod report;
mod versions;
mod ion_diagnostics;
mod log_viewer;
mod server;

fn main() -> anyhow::Result<()> {
    let args: Sgt = Sgt::from_args();

    if cfg!(debug_assertions) {
        println!("Parsed Arguments:");
        println!("{:?}", &args);
        println!();
    }

    // TODO: hide irrelevant tabs, if any, when running remotely. hwconfig? versions?
    match args.command {
        None => gui::run(), // TODO: inject native API
        Some(Command::Backend) => {
            server::main();
            Ok(())
        },
        Some(Command::Frontend) => gui::run(), // TODO: inject HTTP client API
        Some(command) => cli::run(command),
    }
}
