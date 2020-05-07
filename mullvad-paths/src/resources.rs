use std::{env, path::PathBuf};

pub fn get_resource_dir() -> PathBuf {
    match env::var_os("MULLVAD_RESOURCE_DIR") {
        Some(path) => PathBuf::from(path),
        None => get_default_resource_dir(),
    }
}

pub fn get_default_resource_dir() -> PathBuf {
    #[cfg(not(target_os = "android"))]
    {
        match env::current_exe() {
            Ok(mut path) => {
                path.pop();
                path
            }
            Err(e) => {
                log::error!(
                    "Failed finding the install directory. Using working directory: {}",
                    e
                );
                PathBuf::from(".")
            }
        }
    }
    #[cfg(target_os = "android")]
    {
        PathBuf::from(crate::APP_PATH)
    }
}
