use std::{env, path::{PathBuf, Path}, ops::{Deref, DerefMut}};
use talpid_types::ErrorExt;
use tokio::{fs, io};

/// Stores content in a temporary file before moving it to the
/// final destination, ensuring that consumers of the file never
/// end up with partial content. Must be moved with `finalize`.
pub struct AtomicFile {
    file: Option<fs::File>,
    path: PathBuf,
}

impl AtomicFile {
    pub async fn new() -> io::Result<Self> {
        let path = temp_path();
        Ok(Self {
            file: Some(fs::File::create(&path).await?),
            path,
        })
    }

    /// Flushes and moves the file to `target_path`, replacing it if it exists.
    pub async fn finalize(mut self, target_path: &Path) -> io::Result<()> {
        let file = self.file.take().unwrap();

        file.sync_all().await?;
        let std_file = file.into_std().await;
        let _ = tokio::task::spawn_blocking(move || drop(std_file)).await;

        fs::rename(&self.path, target_path).await
    }
}

impl Drop for AtomicFile {
    fn drop(&mut self) {
        // The file will be removed when all file handles are closed
        if let Err(error) = std::fs::remove_file(&self.path) {
            if error.kind() != io::ErrorKind::NotFound {
                log::warn!("{}", error.display_chain_with_msg("Failed to delete AtomicFile"));
            }
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

fn temp_path() -> PathBuf {
    env::temp_dir().join(uuid::Uuid::new_v4().to_string())
}
