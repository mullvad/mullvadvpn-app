use std::path::PathBuf;

pub fn get_data_dir() -> PathBuf {
    std::env::home_dir()
        .expect("No home dir found")
        .join(".local/share/mullvad-release-android")
}
