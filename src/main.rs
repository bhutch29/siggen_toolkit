mod cli;
mod common;
mod events;
mod gui;
mod gui_state;
mod hwconfig;
mod logging;
mod versions;

use cli::*;

fn main() -> anyhow::Result<()> {
    let args: Sgt = Sgt::from_args();

    if cfg!(debug_assertions) {
        println!("Parsed Arguments:");
        println!("{:?}", &args);
    }

    match args.command {
        None => gui::run(),
        Some(command) => cli::run(command),
    }
}
