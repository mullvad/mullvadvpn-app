use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub mod diag;

/// Unable to create new log file
#[derive(thiserror::Error, Debug)]
#[error("Unable to create new log file")]
pub struct RotateLogError(#[from] io::Error);

/// Create a new log file while backing up a previous version of it.
///
/// A new log file is created with the given file name, but if a file with that name already exists
/// it is backed up with the extension changed to `.old.log`.
pub fn rotate_log(file: &Path) -> Result<(), RotateLogError> {
    let backup = file.with_extension("old.log");
    if let Err(error) = LogFile::rename(file, &backup)
        && error.kind() != io::ErrorKind::NotFound
    {
        log::warn!(
            "Failed to rotate log file to {}: {}",
            backup.display(),
            error
        );
    }

    fs::File::create(file).map_err(RotateLogError)?;
    Ok(())
}

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
