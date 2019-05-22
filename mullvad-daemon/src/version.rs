/// A string that identifies the current version of the application
pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

/// Contains the date of the git commit this was built from
pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub fn log_version() {
    log::info!(
        "Starting {} - {} {}",
        env!("CARGO_PKG_NAME"),
        PRODUCT_VERSION,
        COMMIT_DATE,
    )
}
