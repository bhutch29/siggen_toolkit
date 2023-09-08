use crate::hwconfig;
use crate::logging;
use crate::report;
use crate::versions;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "sgt",
    about = "Navigate SigGen deployments.\n\nRun without arguments to launch GUI."
)]
pub struct Sgt {
    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(about = "Run only the backend HTTP server.")]
    Backend,
    #[structopt(about = "Run only the GUI to connect to a running backend.")]
    Frontend,
    #[structopt(name = "hwconfig", about = "Simulated hardware configuration.")]
    HwConfig(HwConfigCommand),
    #[structopt(about = "Logging configuration.")]
    Log(LogCommand),
    #[structopt(about = "Create or browse reports.")]
    Report(ReportCommand),
}

#[derive(StructOpt, Debug)]
pub enum LogCommand {
    Show {},
    Paths,
    SinkPath,
}

#[derive(StructOpt, Debug)]
pub enum ReportCommand {
    Zip {
        name: String,
        #[structopt(short, long, about = "Overwrite file if necessary.")]
        force: bool,
    },
    Upload {
        name: String,
    },
}

#[derive(StructOpt, Debug)]
pub enum HwConfigCommand {
    Restore,
    Show,
    Paths,
}

pub fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::HwConfig(cmd) => match cmd {
            HwConfigCommand::Restore => println!("Not yet implemented!"),
            HwConfigCommand::Show => match hwconfig::read_from(&hwconfig::get_path_or_cwd()) {
                Some(text) => {
                    println!("{}", text)
                }
                None => {
                    println!("No hwconfig found")
                }
            },
            HwConfigCommand::Paths => {
                for path in hwconfig::valid_paths() {
                    println!("{} {}", path.display(), path.exists())
                }
            }
        },
        Command::Log(cmd) => match cmd {
            LogCommand::Show { .. } => {
                logging::show()?;
            }
            LogCommand::Paths => {
                for path in logging::valid_paths() {
                    println!("{} {}", path.display(), path.exists())
                }
            }
            LogCommand::SinkPath => {
                println!("{}", logging::get_log_path_from_current_config().display());
            }
        },
        Command::Report(cmd) => match cmd {
            ReportCommand::Zip { name, force } => {
                let file_name = report::zip_file_name(&name);
                if !force && Path::new(&file_name).exists() {
                    return Err(anyhow::anyhow!(
                        "Destination file already exists: {}\n\
                         Consider using the --force flag or using a unique name.",
                        file_name
                    ));
                }
                println!("{}", file_name);
                report::create_report(&name)?;
            }
            ReportCommand::Upload { name } => {
                let file_name = report::zip_file_name(&name);
                let client = versions::VersionsClient::default();
                let handle = client.upload_report(Path::new(&file_name), None, None)?;
                if handle.join().is_err() {
                    return Err(anyhow::anyhow!("unknown error occurred when uploading"));
                }
                println!(
                    "{}/{}/{}",
                    versions::BASE_FILE_URL,
                    versions::report_segments(),
                    file_name
                );
            }
        },
        _ => {return Err(anyhow::anyhow!("unrecognized command"))}
    };
    Ok(())
}
