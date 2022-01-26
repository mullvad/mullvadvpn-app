use mullvad_types::states::TargetState;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;
use tokio::{fs, io};

const TARGET_START_STATE_FILE: &str = "target-start-state.json";

/// Persists the target state to a file, unless dropped cleanly.
pub struct PersistentTargetState {
    state: TargetState,
    cache_file: PathBuf,
    locked: bool,
}

impl PersistentTargetState {
    /// Initialize using the current target state (or default, if there is none)
    pub async fn new(cache_dir: &Path) -> Self {
        let cache_file = cache_dir.join(TARGET_START_STATE_FILE);
        let mut update_cache = false;
        let state = match fs::read_to_string(&cache_file).await {
            Ok(content) => serde_json::from_str(&content)
                .map(|state| {
                    log::info!(
                        "Loaded cached target state \"{}\" from {}",
                        state,
                        cache_file.display()
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
                    TargetState::Unsecured
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
            cache_file,
            locked: false,
        };
        if update_cache {
            state.save().await;
        }
        state
    }

    /// Override the current target state, if there is one
    pub async fn force(cache_dir: &Path, state: TargetState) -> Self {
        let cache_file = cache_dir.join(TARGET_START_STATE_FILE);
        let state = PersistentTargetState {
            state,
            cache_file,
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
        let _ = fs::remove_file(&self.cache_file).await.map_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Cannot delete target tunnel state cache")
            );
        });
        // prevent the sync destructor from running
        self.locked = true;
    }

    async fn save(&self) {
        log::trace!(
            "Saving tunnel target state to {}",
            self.cache_file.display()
        );
        match serde_json::to_string(&self.state) {
            Ok(data) => {
                if let Err(error) = fs::write(&self.cache_file, data).await {
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
        let _ = std::fs::remove_file(&self.cache_file).map_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Cannot delete target tunnel state cache")
            );
        });
    }
}

impl Deref for PersistentTargetState {
    type Target = TargetState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
