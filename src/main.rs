#![windows_subsystem = "windows"] // Hides console on Windows

mod cli;
mod common;
mod events;
mod gui;
mod gui_state;
mod hwconfig;
mod logging;
mod report;
mod versions;

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
