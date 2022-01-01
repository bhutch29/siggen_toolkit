use crate::{hwconfig, logging, versions};
use std::fmt::Write as fmtWrite;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn create_report(name: &str) -> anyhow::Result<()> {
    let file_name = zip_file_name(name);
    let file = std::fs::File::create(&file_name).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.add_directory("config/", Default::default())?;

    let mut summary = format!("Report Name: {}", name);

    if let Some(version) = versions::installed_version() {
        writeln!(summary, "Installed SigGen Version: {}", version)?;
        zip.start_file("version.txt", Default::default())?;
        zip.write_all(version.as_bytes())?;
    }

    let log_path = logging::get_log_path();
    if log_path.exists() {
        writeln!(summary, "Log File Path: {}", log_path.display())?;
        add_file(&mut zip, log_path)?;
    }

    if let Some(path) = logging::get_path() {
        writeln!(summary, "Log Config Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    if let Some(path) = hwconfig::get_path() {
        writeln!(summary, "Hw Config Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    // TODO: events
    // TODO: exception log

    zip.start_file("summary.txt", Default::default())?;
    zip.write_all(summary.as_bytes())?;

    Ok(())
}

pub fn zip_file_name(name: &str) -> String {
    format!(
        "{}_{}.zip",
        chrono::offset::Local::now().format("%Y-%m-%d"),
        name.replace(char::is_whitespace, "_").to_lowercase(),
    )
}

fn add_file(zip: &mut zip::ZipWriter<std::fs::File>, path: PathBuf) -> anyhow::Result<()> {
    let name = path.file_name().unwrap().to_string_lossy();
    zip.start_file(format!("{}/{}", "config", name), Default::default())?;
    let mut f = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    zip.write_all(&*buffer)?;
    Ok(())
}
