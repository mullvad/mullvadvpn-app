use std::{fs, io, path::Path};

/// Unable to create new log file
#[derive(err_derive::Error, Debug)]
#[error(display = "Unable to create new log file")]
pub struct RotateLogError(#[error(source)] io::Error);

/// Create a new log file while backing up a previous version of it.
///
/// A new log file is created with the given file name, but if a file with that name already exists
/// it is backed up with the extension changed to `.old.log`.
pub fn rotate_log(file: &Path) -> Result<(), RotateLogError> {
    let backup = file.with_extension("old.log");
    if let Err(error) = fs::rename(file, &backup) {
        if error.kind() != io::ErrorKind::NotFound {
            log::warn!(
                "Failed to rotate log file to {}: {}",
                backup.display(),
                error
            );
        }
    }

    fs::File::create(file).map(|_| ()).map_err(RotateLogError)
}
