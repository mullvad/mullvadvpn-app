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
