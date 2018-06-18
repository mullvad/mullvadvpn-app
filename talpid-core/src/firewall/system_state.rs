//! A writer for a blob that would persistently store the system state. Useful
//! for when the application of a secuirty policy proves to be persistent across
//! reboots
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::slice;

const STATE_BACKUP_FILENAME: &str = "system_state_backup";

/// This struct is responsible for saving a binary blob to disk. The binary blob is intended to
/// store system state that should be resotred when the security policy is reset.
pub struct SystemStateWriter {
    /// Full path to the system state backup file
    pub backup_path: Box<Path>,
}

impl SystemStateWriter {
    /// Creates a new SystemStateWriter which will use a file in the cache directory to store system
    /// state that has to be restored.
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        let backup_path = cache_dir
            .as_ref()
            .join(STATE_BACKUP_FILENAME)
            .into_boxed_path();
        Self { backup_path }
    }

    /// Writes a binary blob representing the system state to the backup location before any
    /// security policies are applied.
    pub fn write_backup(&self, data: &[u8]) -> io::Result<()> {
        fs::write(&self.backup_path, &data)
    }

    /// Tries to read a previously saved backup and deletes it after reading it if it exists.
    pub fn consume_state_backup(&self) -> io::Result<Option<Vec<u8>>> {
        match fs::read(&self.backup_path) {
            Ok(blob) => {
                if let Err(e) = self.remove_state_file() {
                    error!("Failed to remove system state backup: {}", e)
                };
                Ok(Some(blob))
            }
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Removes a previously created state backup if it exists.
    pub fn remove_state_file(&self) -> io::Result<()> {
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
    extern crate tempfile;
    use super::*;

    #[test]
    fn can_create_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");

        let mock_system_state: Vec<_> = b"8.8.8.8\n8.8.4.4\n".to_vec();
        let writer = SystemStateWriter::new(&temp_dir);
        writer
            .write_backup(&mock_system_state)
            .expect("failed to write system state");

        let backup = writer
            .consume_state_backup()
            .expect("error when reading system state backup")
            .expect("expected to read system state backup");
        assert_eq!(backup, mock_system_state);

        let empty_read = writer
            .consume_state_backup()
            .expect("error when reading system state backup");
        assert_eq!(empty_read, None);
    }

    #[test]
    fn can_succeed_without_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");

        let writer = SystemStateWriter::new(&temp_dir);
        let backup = writer
            .consume_state_backup()
            .expect("error when reading system state backup");
        assert_eq!(backup, None);
    }

    #[cfg(unix)]
    #[test]
    fn cant_read_without_access() {
        let temp_dir = PathBuf::from("/dev/null");

        let writer = SystemStateWriter::new(&temp_dir);
        let mock_system_state: Vec<_> = b"8.8.8.8\n8.8.4.4\n".to_vec();

        let failure = writer
            .write_backup(&mock_system_state)
            .expect_err("successfully wrote backup file to a directory in /dev/null");
        assert_eq!(failure.kind(), io::ErrorKind::Other);

        let recovery_failure = writer
            .consume_state_backup()
            .expect_err("successfully read backup file in /dev/null");
        assert_eq!(recovery_failure.kind(), io::ErrorKind::Other);
    }

    #[test]
    fn can_remove_when_no_backup_exists() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");

        let writer = SystemStateWriter::new(&temp_dir);
        writer.remove_state_file().expect(
            "Encountered IO error when running remove_state_file when no state file exists",
        );
    }

    #[test]
    fn can_remove_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");
        let writer = SystemStateWriter::new(&temp_dir);
        let mock_system_state = b"8.8.8.8\n8.8.4.4\n".to_vec();

        writer
            .write_backup(&mock_system_state)
            .expect("Failed to write backup");
        writer
            .remove_state_file()
            .expect("Failed to remove state file");

        let empty_backup = writer
            .consume_state_backup()
            .expect("Encountered IO error when no backup file exists");
        assert_eq!(empty_backup, None);
    }

    #[cfg(unix)]
    #[test]
    fn cant_remove_backup_with_io_error() {
        let temp_dir = PathBuf::from("/dev/null");

        let writer = SystemStateWriter::new(&temp_dir);
        let removal_failure = writer
            .remove_state_file()
            .expect_err("successfully removed state file in /dev/null");
        assert_eq!(removal_failure.kind(), io::ErrorKind::Other);
    }
}
