//! A writer for a blob that would persistently store the system state. Useful
//! for when the application of a secuirty policy proves to be persistent across
//! reboots
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

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

    /// Writes a binary blob representing the system DNS settings to the backup location before any
    /// DNS changes are applied.
    pub fn write_backup(&self, data: &[u8]) -> io::Result<()> {
        let mut backup_file = File::create(&self.backup_path)?;
        backup_file.write_all(data)?;
        backup_file.sync_all()
    }

    pub fn read_backup(&self) -> io::Result<Option<Vec<u8>>> {
        match fs::read(&self.backup_path).map(|blob| Some(blob)) {
            Ok(b) => Ok(b),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => Ok(None),
                _ => Err(e),
            },
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

    #[test]
    fn can_create_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");
        let temp_file = temp_dir.path().join("test_file");

        let mock_system_state: Vec<_> = b"8.8.8.8\n8.8.4.4\n".to_vec();
        let writer = SystemStateWriter::new(&temp_file);
        writer
            .write_backup(&mock_system_state)
            .expect("failed to write system state");

        let backup = writer
            .read_backup()
            .expect("error when reading system state backup")
            .expect("expected to read system state backup");
        assert_eq!(backup, mock_system_state);
    }

    #[test]
    fn can_succeed_without_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");
        let temp_file = temp_dir.path().join("test_file");

        let writer = SystemStateWriter::new(&temp_file);
        let backup = writer
            .read_backup()
            .expect("error when reading system state backup");
        assert_eq!(backup, None);
    }

    #[test]
    fn can_remove_when_no_backup_exists() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");
        let temp_file = temp_dir.path().join("test_file");

        let writer = SystemStateWriter::new(&temp_file);
        writer
            .remove_backup()
            .expect("Encountered IO error when running remove_backup when no state file exists");
    }

    #[test]
    fn can_remove_backup() {
        let temp_dir = tempfile::tempdir().expect("failed to crate temp dir");
        let temp_file = temp_dir.path().join("test_file");
        let writer = SystemStateWriter::new(&temp_file);
        let mock_system_state = b"8.8.8.8\n8.8.4.4\n".to_vec();

        writer
            .write_backup(&mock_system_state)
            .expect("Failed to write backup");
        writer.remove_backup().expect("Failed to remove state file");

        let empty_backup = writer
            .read_backup()
            .expect("Encountered IO error when no backup file exists");
        assert_eq!(empty_backup, None);
    }
}
