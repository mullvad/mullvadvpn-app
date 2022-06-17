use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;
use tokio::{fs, io};

/// Stores content in a temporary file before moving it to the
/// final destination, ensuring that consumers of the file never
/// end up with partial content. Must be moved with `finalize`.
pub struct AtomicFile {
    file: Option<fs::File>,
    temp_path: PathBuf,
    target_path: PathBuf,
}

impl AtomicFile {
    pub async fn new(target_path: PathBuf) -> io::Result<Self> {
        let temp_path = target_path.with_file_name(uuid::Uuid::new_v4().to_string());
        Ok(Self {
            file: Some(fs::File::create(&temp_path).await?),
            temp_path,
            target_path,
        })
    }

    /// Flushes and moves the file to `self.target_path`, replacing it if it exists.
    pub async fn finalize(mut self) -> io::Result<()> {
        let file = self.file.take().unwrap();
        let result = Self::finalize_inner(file, &self.temp_path, &self.target_path).await;
        if result.is_err() {
            consume_removal_result(fs::remove_file(&self.temp_path).await);
        }
        result
    }

    async fn finalize_inner(
        file: fs::File,
        temp_path: &Path,
        target_path: &Path,
    ) -> io::Result<()> {
        file.sync_all().await?;
        let std_file = file.into_std().await;
        let _ = tokio::task::spawn_blocking(move || drop(std_file)).await;
        fs::rename(temp_path, target_path).await
    }
}

impl Drop for AtomicFile {
    fn drop(&mut self) {
        if self.file.is_some() {
            log::warn!("Object was not finalized");
            consume_removal_result(std::fs::remove_file(&self.temp_path));
        }
    }
}

fn consume_removal_result(result: io::Result<()>) {
    if let Err(error) = result {
        if error.kind() != io::ErrorKind::NotFound {
            log::warn!(
                "{}",
                error.display_chain_with_msg("Failed to delete temp file")
            );
        }
    }
}

impl Deref for AtomicFile {
    type Target = fs::File;

    fn deref(&self) -> &Self::Target {
        self.file.as_ref().unwrap()
    }
}

impl DerefMut for AtomicFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file.as_mut().unwrap()
    }
}
