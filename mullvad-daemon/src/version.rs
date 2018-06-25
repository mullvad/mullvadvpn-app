
/// Returns a string that identifies the current version of the application
pub fn current() -> &'static str {
    option_env!("MULLVAD_PRODUCT_VERSION").expect("MULLVAD_PRODUCT_VERSION not set")
}

/// Current description returns the current build date
pub fn commit_date() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"))
}
