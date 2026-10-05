use crate::Result;
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

/// Log file with some extra utilities.
pub struct LogFile {
    /// Path to directory of log path.
    directory: PathBuf,
    /// Name of the log file in [`Self::directory`].
    basename: String,
}

impl LogFile {
    pub fn new(name: impl Into<String>, directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            basename: name.into(),
        }
    }

    /// NOTE: `path` must point to a file, not a directory.
    pub fn from_file_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        Self::new(
            path.file_name().unwrap().to_string_lossy(),
            path.parent().unwrap(),
        )
    }

    pub fn with_extension(&mut self, extension: impl AsRef<OsStr>) {
        self.basename
            .push_str(&extension.as_ref().to_string_lossy());
    }

    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let file = Self::from_file_path(from);
        let mut result = Ok(());
        for (path, n) in file.paths_with_index() {
            if let Err(err) = fs::rename(&path, Self::new_file_name(to.as_ref().to_owned(), n)) {
                result = Err(err);
            };
        }
        result
    }

    /// View over files following the Debian convention for naming files.
    /// This will iterate over self.directory/{self.basename,self.basename1, .., self.basenameN}.
    pub fn paths(&self) -> impl Iterator<Item = PathBuf> {
        self.paths_with_index().map(|(path, _)| path)
    }

    /// View over files following the Debian convention for naming files.
    /// This will iterate over self.directory/{self.basename,self.basename1, .., self.basenameN}.
    pub fn paths_with_index(&self) -> impl Iterator<Item = (PathBuf, i32)> {
        let basename = self.directory.join(&self.basename);
        std::iter::once((basename, 0)).chain(
            (1..)
                .map(|n| {
                    let path = self.directory.join(&self.basename);
                    let new_path = Self::new_file_name(path, n);
                    (new_path, n)
                })
                .take_while(|(path, _)| path.exists()),
        )
    }

    fn new_file_name(basename: PathBuf, index: i32) -> PathBuf {
        if index == { 0 } {
            basename
        } else {
            basename.with_extension(index.to_string())
        }
    }
}

/// Creates and returns the logging directory pointed to by `MULLVAD_LOG_DIR`, or the default
/// one if that variable is unset.
pub fn log_dir() -> Result<PathBuf> {
    let permissions = Some(crate::UserPermissions {
        read: true,
        write: false,
        // Unix: Make directory contents readable
        execute: cfg!(unix),
    });

    crate::create_dir(get_log_dir()?, permissions)
}

/// Get the logging directory, but don't try to create it.
pub fn get_log_dir() -> Result<PathBuf> {
    match env::var_os("MULLVAD_LOG_DIR") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_log_dir(),
    }
}

#[cfg(unix)]
pub fn get_default_log_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("/var/log").join(crate::PRODUCT_NAME);
    Ok(dir)
}

#[cfg(windows)]
pub fn get_default_log_dir() -> Result<PathBuf> {
    let dir = crate::windows::get_allusersprofile_dir()?.join(crate::PRODUCT_NAME);
    Ok(dir)
}

/// Returns the directory where any Mullvad-related frontend stores its logs.
/// Logging to this folder does not require root-permissions.
#[cfg(not(target_os = "android"))]
pub fn frontend_log_dir() -> Result<PathBuf> {
    use crate::Error;
    let mullvad_dir = cfg_select! {
        any(target_os = "linux", target_os = "windows") => { "Mullvad VPN/logs" }
        target_os = "macos" => { "Library/Logs/Mullvad VPN" }
    };
    local_log_dir()
        .map(|config_dir| config_dir.join(mullvad_dir))
        .map_err(Error::FindDirError)
}

fn local_log_dir() -> std::io::Result<PathBuf> {
    use std::io;
    // NOTE: These invocations to different `dirs` functions is due to Electron's app.getPath API:
    // https://www.electronjs.org/docs/latest/api/app#appgetpathname.
    cfg_select! {
        target_os = "linux" => { dirs::config_local_dir() }
        target_os = "macos" => { dirs::home_dir() }
        target_os = "windows" => { dirs::data_local_dir() }
    }
    .ok_or(io::Error::from(io::ErrorKind::NotFound))
}
