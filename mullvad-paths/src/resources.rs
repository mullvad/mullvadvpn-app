use std::env;
use std::path::PathBuf;

pub fn get_resource_dir() -> PathBuf {
    match env::var_os("MULLVAD_RESOURCE_DIR") {
        Some(path) => PathBuf::from(path),
        None => get_default_resource_dir(),
    }
}

fn get_default_resource_dir() -> PathBuf {
    match env::current_exe() {
        Ok(mut path) => {
            path.pop();
            path
        }
        Err(e) => {
            error!(
                "Failed finding the install directory. Using working directory: {}",
                e
            );
            PathBuf::from(".")
        }
    }
}
