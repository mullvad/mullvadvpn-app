//! The WireGuard identity of this installation.
//!
//! This module replaces what used to be the account and device handling. There is no account,
//! no login, no device list, no key rotation and no expiry check. The only state is a
//! [`WireguardIdentity`]: the private key and tunnel addresses taken from a WireGuard config file
//! generated at mullvad.net. It is persisted to a file in the settings directory and handed out
//! to the tunnel parameter generator. Nothing in here talks to the network.

use mullvad_types::wireguard::{
    ConfigParseError, WireguardIdentity, WireguardIdentityInfo,
};
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use talpid_error::ErrorExt;
use tokio::sync::RwLock;

/// File in the settings directory that stores the identity.
pub const IDENTITY_FILENAME: &str = "wireguard-identity.json";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to read or write the WireGuard identity")]
    Io(#[from] io::Error),
    #[error("Failed to serialize the WireGuard identity")]
    Serialize(#[from] serde_json::Error),
    #[error("Invalid WireGuard config")]
    InvalidConfig(#[from] ConfigParseError),
}

/// Handle to the persisted [`WireguardIdentity`]. Cheap to clone.
#[derive(Clone)]
pub struct IdentityHandle {
    inner: Arc<Inner>,
}

struct Inner {
    path: PathBuf,
    state: RwLock<Option<WireguardIdentity>>,
}

impl IdentityHandle {
    /// Load the identity from the settings directory. A missing file means no identity. A file
    /// that cannot be parsed is moved out of the way, and is treated as no identity.
    pub async fn load(settings_dir: &Path) -> Result<Self, Error> {
        let path = settings_dir.join(IDENTITY_FILENAME);
        let identity = match tokio::fs::read(&path).await {
            Ok(bytes) if bytes.iter().all(u8::is_ascii_whitespace) => None,
            Ok(bytes) => match serde_json::from_slice::<WireguardIdentity>(&bytes) {
                Ok(identity) => Some(identity),
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "The stored WireGuard identity is corrupt. Ignoring it"
                        )
                    );
                    let corrupt = path.with_extension("json.corrupt");
                    if let Err(error) = tokio::fs::rename(&path, &corrupt).await {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to move aside corrupt identity")
                        );
                    }
                    None
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(Error::Io(error)),
        };
        Ok(IdentityHandle {
            inner: Arc::new(Inner {
                path,
                state: RwLock::new(identity),
            }),
        })
    }

    /// Get the current identity, including the private key.
    pub async fn get(&self) -> Option<WireguardIdentity> {
        self.inner.state.read().await.clone()
    }

    /// Get the public details of the current identity.
    pub async fn info(&self) -> Option<WireguardIdentityInfo> {
        self.inner
            .state
            .read()
            .await
            .as_ref()
            .map(WireguardIdentityInfo::from)
    }

    /// Parse a WireGuard config file and make it the current identity, replacing any previous
    /// one. The identity is persisted before this returns.
    pub async fn import_config(&self, config: &str) -> Result<WireguardIdentityInfo, Error> {
        let identity = WireguardIdentity::from_config(config)?;
        let info = WireguardIdentityInfo::from(&identity);

        let mut state = self.inner.state.write().await;
        write_identity(&self.inner.path, &identity).await?;
        *state = Some(identity);
        Ok(info)
    }

    /// Forget the current identity and delete it from disk. Returns whether there was one.
    pub async fn clear(&self) -> Result<bool, Error> {
        let mut state = self.inner.state.write().await;
        let had_identity = state.is_some();
        match tokio::fs::remove_file(&self.inner.path).await {
            Ok(()) => (),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(Error::Io(error)),
        }
        *state = None;
        Ok(had_identity)
    }
}

/// Atomically write the identity to `path`, readable by the owner only where that can be
/// expressed.
async fn write_identity(path: &Path, identity: &WireguardIdentity) -> Result<(), Error> {
    let data = serde_json::to_vec_pretty(identity)?;
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        use io::Write;

        let temp_path = path.with_extension("json.tmp");
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp_path, &path)
    })
    .await
    .map_err(io::Error::other)??;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use talpid_types::net::wireguard::PrivateKey;

    fn config_with(key: &PrivateKey) -> String {
        format!(
            "[Interface]\nPrivateKey = {}\nAddress = 10.64.12.34/32,fc00:bbbb:bbbb:bb01::1:2/128\n",
            key.to_base64()
        )
    }

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn no_file_means_no_identity() {
        let guard = temp_dir();
        let dir = guard.path().to_path_buf();
        let handle = IdentityHandle::load(&dir).await.unwrap();
        assert!(handle.get().await.is_none());
        assert!(handle.info().await.is_none());
        assert!(!handle.clear().await.unwrap());
    }

    #[tokio::test]
    async fn import_persists_and_survives_restart() {
        let guard = temp_dir();
        let dir = guard.path().to_path_buf();
        let key = PrivateKey::new_from_random();

        let handle = IdentityHandle::load(&dir).await.unwrap();
        let info = handle.import_config(&config_with(&key)).await.unwrap();
        assert_eq!(info.public_key.key, key.public_key());
        assert_eq!(info.ipv4_address, "10.64.12.34/32".parse().unwrap());

        let reloaded = IdentityHandle::load(&dir).await.unwrap();
        let identity = reloaded.get().await.expect("identity was persisted");
        assert_eq!(identity.private_key.public_key(), key.public_key());
        assert_eq!(reloaded.info().await.unwrap(), info);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.join(IDENTITY_FILENAME))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o077, 0, "identity must not be accessible by others");
        }
    }

    #[tokio::test]
    async fn importing_again_replaces_and_clear_deletes() {
        let guard = temp_dir();
        let dir = guard.path().to_path_buf();
        let first = PrivateKey::new_from_random();
        let second = PrivateKey::new_from_random();

        let handle = IdentityHandle::load(&dir).await.unwrap();
        handle.import_config(&config_with(&first)).await.unwrap();
        handle.import_config(&config_with(&second)).await.unwrap();
        assert_eq!(
            handle.get().await.unwrap().private_key.public_key(),
            second.public_key()
        );

        assert!(handle.clear().await.unwrap());
        assert!(handle.get().await.is_none());
        assert!(!dir.join(IDENTITY_FILENAME).exists());
    }

    #[tokio::test]
    async fn invalid_config_leaves_identity_untouched() {
        let guard = temp_dir();
        let dir = guard.path().to_path_buf();
        let key = PrivateKey::new_from_random();
        let handle = IdentityHandle::load(&dir).await.unwrap();
        handle.import_config(&config_with(&key)).await.unwrap();

        let error = handle.import_config("[Interface]\nPrivateKey = nope\n").await;
        assert!(matches!(error, Err(Error::InvalidConfig(_))));
        assert_eq!(
            handle.get().await.unwrap().private_key.public_key(),
            key.public_key()
        );
    }

    #[tokio::test]
    async fn corrupt_file_is_set_aside() {
        let guard = temp_dir();
        let dir = guard.path().to_path_buf();
        std::fs::write(dir.join(IDENTITY_FILENAME), b"{ definitely not json").unwrap();
        let handle = IdentityHandle::load(&dir).await.unwrap();
        assert!(handle.get().await.is_none());
        assert!(!dir.join(IDENTITY_FILENAME).exists());
        assert!(dir.join("wireguard-identity.json.corrupt").exists());
    }
}
