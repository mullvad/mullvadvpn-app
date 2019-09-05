//! A writer for a blob that would persistently store the system state. Useful
//! for when the application of a secuirty policy proves to be persistent across
//! reboots
use std::{fs, io, path::Path};

/// This struct is responsible for saving a binary blob to disk. The binary blob is intended to
/// store system DNS settings that should be restored when the DNS settings are reset.
pub struct SystemStateWriter {
    /// Full path to the system state backup file
    pub backup_path: Box<Path>,
}

impl SystemStateWriter {
    /// Creates a new SystemStateWriter which will use a file in the cache directory to store system
    /// DNS state that has to be restored.
    pub fn new<P: AsRef<Path>>(backup_path: P) -> Self {
        Self {
            backup_path: backup_path.as_ref().to_owned().into_boxed_path(),
        }
    }

    /// Removes a previously created state backup if it exists.
    pub fn remove_backup(&self) -> io::Result<()> {
        match fs::remove_file(&self.backup_path) {
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    Err(e)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::{self, File};
    use std::io::prelude::*;

    #[test]
    fn can_remove_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let temp_file = temp_dir.path().join("test_file");

        let mut file_handle = File::create(&temp_file).expect("failed to create dummy backup file");
        file_handle
            .write_all(b"Hello, world!")
            .expect("failed to write to dummy backup file");

        let writer = SystemStateWriter::new(&temp_file);
        writer
            .remove_backup()
            .expect("failed to remove backup file");
    }

    #[test]
    fn can_remove_when_no_backup_exists() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let temp_file = temp_dir.path().join("test_file");

        let writer = SystemStateWriter::new(&temp_file);
        writer
            .remove_backup()
            .expect("Encountered IO error when running remove_backup when no state file exists");
    }
}
