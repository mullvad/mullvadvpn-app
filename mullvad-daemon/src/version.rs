/// Contains the date of the git commit this was built from
pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub fn is_beta_version() -> bool {
    mullvad_version::VERSION.contains("beta")
}

pub fn log_version() {
    log::info!(
        "Starting {} - {} {}",
        env!("CARGO_PKG_NAME"),
        mullvad_version::VERSION,
        COMMIT_DATE,
    )
}
