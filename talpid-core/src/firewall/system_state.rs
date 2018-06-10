/// A writer for a blob that would persistently store the system state. Useful
/// for when the application of a secuirty policy proves to be persistent across
/// reboots
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::slice;

const STATE_BACKUP_FILENAME: &str = "system_state_backup";

/// An FFI callback for writing system state data
pub extern "system" fn write_system_state_backup_cb(
    blob: *const u8,
    length: u32,
    state_writer: *mut SystemStateWriter,
) -> i32 {
    if state_writer.is_null() {
        error!("State writer pointer is null, can't save system state backup");
        return -1;
    }

    unsafe {
        trace!(
            "Writing {} bytes to store system state backup to {}",
            length,
            (*state_writer).backup_path.to_string_lossy()
        );
        let data = slice::from_raw_parts(blob, length as usize);
        match (*state_writer).write_backup(data) {
            Ok(()) => 0,
            Err(e) => {
                error!(
                    "Failed to write system state backup to {} because {}",
                    (*state_writer).backup_path.to_string_lossy(),
                    e
                );
                e.raw_os_error().unwrap_or(-1)
            }
        }
    }
}

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

    /// Writes a binary blob representing the system state before any security policies were
    /// applied to a specified backup location.
    pub fn write_backup(&self, data: &[u8]) -> io::Result<()> {
        fs::write(&self.backup_path, &data)
    }

    /// Tries to read a previously saved backup and delete's it after reading it if it exists.
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
        writer
            .remove_state_file()
            .expect("Failed to remove state file when one did not exist");
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
            .expect("Failed to remove state file when one did not exist");
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
