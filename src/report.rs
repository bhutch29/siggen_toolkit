use crate::{hwconfig, logging, versions};
use std::fmt::Write as fmtWrite;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn create_report(name: &str) -> anyhow::Result<()> {
    let file_name = zip_file_name(name);
    let file = std::fs::File::create(&file_name).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.add_directory("config/", Default::default())?;

    let mut summary = format!("Report Name: {}\n", name);

    if let Some(version) = versions::installed_version() {
        writeln!(summary, "Installed SigGen Version: {}", version)?;
        zip.start_file("version.txt", Default::default())?;
        zip.write_all(version.as_bytes())?;
    }

    let path = logging::get_log_path();
    if path.exists() {
        writeln!(summary, "Log File Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    let path = logging::get_exception_log_path();
    if path.exists() {
        writeln!(summary, "Exception Log File Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    if let Some(path) = logging::get_path() {
        writeln!(summary, "Log Config Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    let path = get_no_reset_system_settings_path();
    if path.exists() {
        writeln!(summary, "No Reset System Settings Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    let user_settings_paths = get_all_user_settings_paths();
    if !user_settings_paths.is_empty() {
        writeln!(summary, "Per-User Settings Path: {}", user_settings_paths.join(", "))?;
        for path in user_settings_paths {
            add_file(&mut zip, PathBuf::from(path))?;
        }
    }

    if let Some(path) = hwconfig::get_path() {
        writeln!(summary, "Hw Config Path: {}", path.display())?;
        add_file(&mut zip, path)?;
    }

    // TODO: events

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

pub fn get_no_reset_system_settings_path() -> PathBuf {
    PathBuf::from("E:\\")
        .join("Keysight")
        .join("PathWave")
        .join("SignalGenerator")
        .join("SigGenInstrumentSpecificSettings.sgen")
}

pub fn get_all_user_settings_paths() -> Vec<String> {
    dirs::data_dir()
        .and_then(|dir| {
            glob::glob(
                dir.join("Keysight")
                    .join("PathWave")
                    .join("SignalGenerator")
                    .join("*.sgen")
                    .to_string_lossy()
                    .as_ref(),
            )
            .ok()
        })
        .map(|glob| glob.flatten().map(|path| path.to_string_lossy().to_string()).collect())
        .unwrap_or_default()
}
