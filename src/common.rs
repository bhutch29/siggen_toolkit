use std::path::{Path, PathBuf};
use std::process;

// TODO: backend
pub fn in_cwd<P: AsRef<Path>>(file: P) -> PathBuf {
    std::env::current_dir().unwrap().join(file)
}

// TODO: backend
pub fn open_explorer(path: &Path) -> anyhow::Result<()> {
    process::Command::new(if cfg!(windows) { "explorer" } else { "xdg-open" })
        .arg(path.parent().unwrap())
        .spawn()?;
    Ok(())
}
