/// Returns a string that identifies the current version of the application
pub fn current() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/git-commit-desc.txt"))
}

/// Current description returns the current build date
pub fn commit_date() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"))
}
