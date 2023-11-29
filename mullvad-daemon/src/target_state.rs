use mullvad_types::states::TargetState;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;
use tokio::{fs, io};

/// State to use by default if there is no cache.
const DEFAULT_TARGET_STATE: TargetState = TargetState::Unsecured;
const TARGET_START_STATE_FILE: &str = "target-start-state.json";

/// Persists the target state to a file, which is only removed if the instance is dropped cleanly.
pub struct PersistentTargetState {
    state: TargetState,
    cache_path: PathBuf,
    locked: bool,
}

impl PersistentTargetState {
    /// Initialize using the current target state (if there is one)
    pub async fn new(cache_dir: &Path) -> Self {
        let cache_path = cache_dir.join(TARGET_START_STATE_FILE);
        let mut update_cache = false;
        let state = match fs::read_to_string(&cache_path).await {
            Ok(content) => serde_json::from_str(&content)
                .map(|state| {
                    log::info!(
                        "Loaded cached target state \"{}\" from {}",
                        state,
                        cache_path.display()
                    );
                    state
                })
                .unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to parse cached target tunnel state")
                    );
                    update_cache = true;
                    TargetState::Secured
                }),
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    log::debug!("No cached target state to load");
                    DEFAULT_TARGET_STATE
                } else {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to read cached target tunnel state")
                    );
                    update_cache = true;
                    TargetState::Secured
                }
            }
        };
        let state = PersistentTargetState {
            state,
            cache_path,
            locked: false,
        };
        if update_cache {
            state.save().await;
        }
        state
    }

    /// Override the current target state, if there is one
    pub async fn force(cache_dir: &Path, state: TargetState) -> Self {
        let cache_path = cache_dir.join(TARGET_START_STATE_FILE);
        let state = PersistentTargetState {
            state,
            cache_path,
            locked: false,
        };
        state.save().await;
        state
    }

    pub async fn set(&mut self, new_state: TargetState) {
        if new_state != self.state {
            self.state = new_state;
            self.save().await;
        }
    }

    /// Prevent the file from being removed when the instance is dropped.
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Async destructor
    pub async fn finalize(mut self) {
        if self.locked {
            return;
        }
        let _ = fs::remove_file(&self.cache_path).await.map_err(|error| {
            if error.kind() != io::ErrorKind::NotFound {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Cannot delete target tunnel state cache")
                );
            }
        });
        // prevent the sync destructor from running
        self.locked = true;
    }

    async fn save(&self) {
        log::trace!(
            "Saving tunnel target state to {}",
            self.cache_path.display()
        );
        match serde_json::to_string(&self.state) {
            Ok(data) => {
                if let Err(error) = fs::write(&self.cache_path, data).await {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to write cache target state")
                    );
                }
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to serialize cache target state")
                )
            }
        }
    }
}

impl Drop for PersistentTargetState {
    fn drop(&mut self) {
        if self.locked {
            return;
        }
        let _ = std::fs::remove_file(&self.cache_path).map_err(|error| {
            if error.kind() != io::ErrorKind::NotFound {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Cannot delete target tunnel state cache")
                );
            }
        });
    }
}

impl Deref for PersistentTargetState {
    type Target = TargetState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// A temporary directory which is automatically removed when dropped.
    struct TestCache {
        path: PathBuf,
    }

    #[derive(err_derive::Error, Debug)]
    enum Error {
        #[error(display = "IO Error")]
        Io(#[error(source)] io::Error),
        #[error(display = "Error while Serializing/Deserializing")]
        Serde(#[error(source)] serde_json::Error),
    }

    impl TestCache {
        fn new() -> Self {
            let path = Self::tmp_cache_dir();
            std::fs::create_dir_all(&path).expect("Could not create temporary directory {path}");
            TestCache { path }
        }

        fn next_nonce() -> usize {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static NONCE: AtomicUsize = AtomicUsize::new(0);
            NONCE.fetch_add(1, Ordering::Relaxed)
        }

        /// Generate a temporary directory. Necessary to test logic involving
        /// filesystem I/O.
        fn tmp_cache_dir() -> PathBuf {
            std::env::temp_dir().join("mullvad").join(format!(
                "target-state-cache-{nonce}",
                nonce = Self::next_nonce()
            ))
        }

        async fn write<C>(&self, contents: C)
        where
            C: AsRef<[u8]>,
        {
            let target_state_cache = self.path.join(TARGET_START_STATE_FILE);
            tokio::fs::write(target_state_cache, contents)
                .await
                .expect(&format!("Failed to write bytes to cache"));
        }

        async fn save_target_state(&self, target_state: &TargetState) {
            self.write(
                serde_json::to_string(target_state)
                    .expect("Could not serialize target state {target_state}"),
            )
            .await
        }

        async fn read_target_state(&self) -> Result<TargetState, Error> {
            let target_state_cache = self.path.join(TARGET_START_STATE_FILE);
            let file_content = tokio::fs::read_to_string(target_state_cache).await?;
            Ok(serde_json::from_str(&file_content)?)
        }
    }

    impl Deref for TestCache {
        type Target = PathBuf;

        fn deref(&self) -> &Self::Target {
            &self.path
        }
    }

    impl Drop for TestCache {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir(&self.path).map_err(|error| {
                if error.kind() != io::ErrorKind::NotFound {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Cannot delete target tunnel state cache")
                    );
                }
            });
        }
    }

    /// If no target state cache exist, the default target state is used. This
    /// is the most basic check.
    #[tokio::test]
    async fn test_target_state_initialization_empty() {
        // A completely blank slate. No target state cache file has been created yet.
        let cache_dir = TestCache::new();
        let target_state = PersistentTargetState::new(&cache_dir).await;
        assert_eq!(*target_state, DEFAULT_TARGET_STATE);
    }

    /// If a target state cache exist with some target state, the state can be
    /// read-back successfully.
    #[tokio::test]
    async fn test_target_state_initialization_existing() {
        let cache_dir = TestCache::new();
        for cached_state in [TargetState::Secured, TargetState::Unsecured] {
            // Create a target state cache file before initializing `PersistentTargetState`.
            cache_dir.save_target_state(&cached_state).await;
            let target_state = PersistentTargetState::new(&cache_dir).await;
            assert_eq!(*target_state, cached_state);
        }
    }

    /// The state can not be read-back successfully if the state file has become
    /// corrupt. In such cases, initializing a [`PersistentTargetState`] should
    /// yield a "better safe than sorry"-target state of `Secured`.
    #[tokio::test]
    async fn test_target_corrupt_state_cache() {
        let cache_dir = TestCache::new();
        // No previous target state cache has been created. Thus `target_state`
        // will be `Unsecured`.
        let mut target_state = PersistentTargetState::new(&cache_dir).await;
        target_state.save().await;
        // Lock the file, indicating that the target state cache should be
        // preserved on disk when dropped.
        target_state.lock();
        drop(target_state);
        // Ensure that it it still exists on disk.
        let target_state_cache = cache_dir.read_target_state().await;
        match target_state_cache {
            Ok(target_state) => assert_eq!(target_state, TargetState::Unsecured),
            Err(_) => {
                panic!("Failed to read target state cache");
            }
        }
        // Intentionally corrupt the target state cache.
        cache_dir.write("Not a valid target state").await;
        // Reading back a corrupt target state cache should yield
        // `TargetState::Secured`.
        let target_state = PersistentTargetState::new(&cache_dir).await;
        assert_eq!(*target_state, TargetState::Secured);
    }

    /// [`PersistentTargetState`] implements [`Drop`] to conditionally persist
    /// the current target state to disk during the destruction of a
    /// [`PersistentTargetState`] value.
    ///
    /// The current target state should be persisted to disk if the daemon can
    /// not drop the [`PersistentTargetState`] cleanly Otherwise, the target
    /// state cache is not critical to prevent leaking during app startup, and
    /// may therefore be removed.
    #[tokio::test]
    async fn test_drop() {
        let cache_dir = TestCache::new();
        let target_state = PersistentTargetState::new(&cache_dir).await;
        target_state.save().await;
        drop(target_state);
        match cache_dir.read_target_state().await {
            Err(Error::Io(err)) => {
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
            }
            Ok(_) | Err(_) => {
                panic!("The target state was not dropped cleanly, but it should have been")
            }
        };

        // Lock the file, indicating that it should not be removed upon drop.
        let mut target_state = PersistentTargetState::new(&cache_dir).await;
        target_state.lock();
        target_state.save().await;
        drop(target_state);
        let cached_target_state = cache_dir.read_target_state().await;
        assert!(
            cached_target_state.is_ok(),
            "Could not read target state. It was errounusly removed."
        );
    }
}
