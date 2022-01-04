fn main() -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    winres::WindowsResource::new()
        .set_icon("keysight-logo-gear.ico")
        .compile()?;
    Ok(())
}