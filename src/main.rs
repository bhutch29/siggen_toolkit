// TODO: This works to hide the console when launching the GUI but then hides the console output when using the CLI
// #![windows_subsystem = "windows"] // Hides console on Windows

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

use cli::*;

fn main() -> anyhow::Result<()> {
    let args: Sgt = Sgt::from_args();

    if cfg!(debug_assertions) {
        println!("Parsed Arguments:");
        println!("{:?}", &args);
        println!();
    }

    match args.command {
        None => gui::run(),
        Some(command) => cli::run(command),
    }
}
