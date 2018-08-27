use std::path::Path;
use std::{fs, io};

error_chain!{}

/// Create a new log file while backing up a previous version of it.
///
/// A new log file is created with the given file name, but if a file with that name already exists
/// it is backed up with the extension changed to `.old.log`.
pub fn rotate_log(file: &Path) -> Result<()> {
    let backup = file.with_extension("old.log");
    fs::rename(file, backup).unwrap_or_else(|error| {
        if error.kind() != io::ErrorKind::NotFound {
            warn!("Failed to rotate log file ({})", error);
        }
    });

    fs::File::create(file).chain_err(|| "Unable to create new log file")?;
    Ok(())
}
