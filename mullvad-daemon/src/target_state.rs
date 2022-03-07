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
