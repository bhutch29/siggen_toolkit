use crate::{hwconfig, logging, versions};
use std::fmt::Write as fmtWrite;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip;

pub fn create_report(file_name: &String) -> anyhow::Result<()> {
    let file = std::fs::File::create(&file_name).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.add_directory("config/", Default::default())?;

    let mut summary = String::new();

    let log_path = logging::get_log_path();
    if log_path.exists() {
        writeln!(summary, "Log File: {}", log_path.display())?;
        add_file(&mut zip, log_path)?;
    }

    if let Some(path) = logging::get_path() {
        writeln!(summary, "Log Config File: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    if let Some(path) = hwconfig::get_path() {
        writeln!(summary, "Hw Config File: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    if let Some(version) = versions::installed_version() {
        writeln!(summary, "Installed SigGen Version: {}", version)?;
        zip.start_file("version.txt", Default::default())?;
        zip.write_all(version.as_bytes())?;
    }

    // TODO: events

    zip.start_file("summary.txt", Default::default())?;
    zip.write_all(summary.as_bytes())?;

    Ok(())
}

pub fn zip_file_name(name: &String, force: bool) -> anyhow::Result<String> {
    let file_name = format!(
        "report_{}_{}.zip",
        name,
        chrono::offset::Local::now().format("%Y-%m-%d")
    );
    if !force && Path::new(&file_name).exists() {
        return Err(anyhow::anyhow!(
            "Destination file already exists: {}\n\
             Consider using the --force flag or using a unique name.",
            file_name
        ));
    }
    Ok(file_name)
}

fn add_file(zip: &mut zip::ZipWriter<std::fs::File>, path: PathBuf) -> anyhow::Result<()> {
    let name = path.file_name().unwrap();
    println!("Adding file {:?} as {:?} ...", path, name);
    zip.start_file(
        format!("{}/{}", "config", name.to_string_lossy()),
        Default::default(),
    )?;
    let mut f = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    zip.write_all(&*buffer)?;
    Ok(())
}
