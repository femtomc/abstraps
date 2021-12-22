use color_eyre::Report;

/// Diagnostics setup for tracing and error reporting.
pub fn diagnostics_setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt().init();

    Ok(())
}

/////
///// Locations
/////

#[derive(Debug, Hash)]
pub enum LocationInfo {
    Unknown,
    FileLineCol(String, usize, usize),
    NameFileLineCol(String, String, usize, usize),
    InlinedFrom(Vec<LocationInfo>),
}
