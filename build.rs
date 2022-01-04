fn main() -> std::io::Result<()> {
    // TODO: fails due to missing-file error on Windows
    // #[cfg(target_os = "windows")]
    // winres::WindowsResource::new()
    //     .set_icon("keysight-logo-gear.ico")
    //     .compile()?;
    Ok(())
}
