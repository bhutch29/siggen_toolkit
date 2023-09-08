// TODO: This works to hide the console when launching the GUI but then hides the console output when using the CLI
// #![windows_subsystem = "windows"] // Hides console on Windows

use crate::cli::{Command, Sgt};
use model::{NativeModel, HttpClientModel};
use structopt::StructOpt;

mod cli;
mod common;
mod gui;
mod gui_state;
mod hwconfig;
mod ion_diagnostics;
mod log_viewer;
mod logging;
mod report;
mod server;
mod versions;
mod model;

fn main() -> anyhow::Result<()> {
    let args: Sgt = Sgt::from_args();

    if cfg!(debug_assertions) {
        println!("Parsed Arguments:");
        println!("{:?}", &args);
        println!();
    }

    match args.command {
        None => gui::run(Box::new(NativeModel::default())),
        Some(Command::Backend) => {
            server::main();
            Ok(())
        }
        Some(Command::Frontend) => gui::run(Box::new(HttpClientModel::default())),
        Some(command) => cli::run(command),
    }
}
