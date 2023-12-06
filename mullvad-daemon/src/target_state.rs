use mullvad_types::states::TargetState;
use std::{
    future::Future,
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
        let TargetStateInner {
            state,
            update_cache,
        } = Self::read_target_state(&cache_path, fs::read_to_string).await;
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

    /// Construct a [`TargetState`] from cache.
    ///
    /// `read_cache` allows the caller to decide how to read from a cache of
    /// [`TargetState`].
    ///
    /// This function will always succeed, even in the presence of IO
    /// operations. Errors are handled gracefully by defaulting to safe target
    /// states if necessary.
    async fn read_target_state<F, R>(cache: &Path, read_cache: F) -> TargetStateInner
    where
        F: FnOnce(PathBuf) -> R,
        R: Future<Output = io::Result<String>>,
    {
        match read_cache(cache.to_path_buf()).await {
            Ok(content) => serde_json::from_str(&content)
                .map(|state| {
                    log::info!(
                        "Loaded cached target state \"{}\" from {}",
                        state,
                        cache.display()
                    );
                    TargetStateInner {
                        state,
                        update_cache: false,
                    }
                })
                .unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to parse cached target tunnel state")
                    );
                    TargetStateInner {
                        state: TargetState::Secured,
                        update_cache: true,
                    }
                }),

            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                log::debug!("No cached target state to load");
                TargetStateInner {
                    state: DEFAULT_TARGET_STATE,
                    update_cache: false,
                }
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to read cached target tunnel state")
                );
                TargetStateInner {
                    state: TargetState::Secured,
                    update_cache: true,
                }
            }
        }
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

/// The result of calling `read_target_state`.
struct TargetStateInner {
    state: TargetState,
    /// In some circumstances, the target state cache should be updated on disk
    /// upon initialization a [`PersistentTargetState`]. This is signaled to the
    /// constructor of [`PersistentTargetState`] by setting this value to
    /// `true`.
    update_cache: bool,
}

impl Deref for TargetStateInner {
    type Target = TargetState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static DUMMY_CACHE_DIR: &str = "target-state-test";

    /// If no target state cache exist, the default target state is used. This
    /// is the most basic check.
    #[tokio::test]
    async fn test_target_state_initialization_empty() {
        let target_state =
            PersistentTargetState::read_target_state(Path::new(DUMMY_CACHE_DIR), |_| async {
                // A completely blank slate. No target state cache file has been created yet.
                Err(io::ErrorKind::NotFound.into())
            })
            .await;
        assert_eq!(*target_state, DEFAULT_TARGET_STATE);
    }

    /// If a target state cache exist with some target state, the state can be
    /// read-back successfully.
    #[tokio::test]
    async fn test_target_state_initialization_existing() {
        for cached_state in [TargetState::Secured, TargetState::Unsecured] {
            let target_state =
                PersistentTargetState::read_target_state(Path::new(DUMMY_CACHE_DIR), |_| async {
                    Ok(serde_json::to_string(&cached_state).unwrap())
                })
                .await;
            assert_eq!(*target_state, cached_state);
        }
    }

    /// The state can not be read-back successfully if the state file has become
    /// corrupt. In such cases, initializing a [`PersistentTargetState`] should
    /// yield a "better safe than sorry"-target state of `Secured`.
    #[tokio::test]
    async fn test_target_corrupt_state_cache() {
        let target_state =
            PersistentTargetState::read_target_state(Path::new(DUMMY_CACHE_DIR), |_| async {
                // Intentionally corrupt the target state cache.
                Ok("Not a valid target state".to_string())
            })
            .await;
        // Reading back a corrupt target state cache should yield `TargetState::Secured`.
        assert_eq!(*target_state, TargetState::Secured);
    }
}
