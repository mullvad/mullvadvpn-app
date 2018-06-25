/// A string that identifies the current version of the application
pub static CURRENT: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

/// Contains the date of the git commit this was built from
pub static COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));
