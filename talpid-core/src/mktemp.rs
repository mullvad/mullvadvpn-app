use std::env;
use std::path::{Path, PathBuf};
use std::fs;

use uuid::Uuid;

#[derive(Debug)]
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    /// Create a new unique `TempFile`. The file will not exist after this.
    pub fn new() -> Self {
        TempFile {
            path: generate_path(),
        }
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.path.clone()
    }
}

impl AsRef<Path> for TempFile {
    fn as_ref(&self) -> &Path {
        &self.path.as_path()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            error!("Unable to remove TempFile {}: {:?}", self.path.to_string_lossy(), e);
        }
    }
}

fn generate_path() -> PathBuf {
    env::temp_dir().join(Uuid::new_v4().to_string())
}
