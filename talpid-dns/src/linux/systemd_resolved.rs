use std::fmt;
use std::fs::{self, canonicalize, read_link};
use std::io;
use std::net::IpAddr;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use log::{trace, warn};
use safelog::Sensitive;
use talpid_dbus::systemd_resolved::dbus_rs::{AsyncHandle, SystemdResolved as DbusInterface};
use talpid_error::ErrorExt;

pub(crate) use talpid_dbus::systemd_resolved::dbus_rs::Error as SystemdDbusError;

use crate::imp::interface::{IfaceIndexLookupError, iface_index};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("systemd-resolved operation failed {0}")]
    SystemdResolved(#[from] SystemdDbusError),

    #[error("Failed to resolve interface index with error {0}")]
    InterfaceName(#[from] IfaceIndexLookupError),

    #[error("/etc/resolv.conf is not managed by systemd-resolved {0}")]
    ResolvConf(#[from] io::Error),
}

pub struct SystemdResolved {
    pub dbus_interface: AsyncHandle,
    tunnel_index: u32,
    backup: Option<Backup>,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            tunnel_index: 0,
            backup: Backup::new()?,
        };

        Ok(systemd_resolved)
    }

    pub async fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let tunnel_index = iface_index(interface_name)?;
        self.tunnel_index = tunnel_index;

        if let Err(error) = self.dbus_interface.disable_dot(self.tunnel_index).await {
            log::error!("Failed to disable DoT: {}", error.display_chain());
        }

        if let Err(error) = self
            .dbus_interface
            .set_domains(tunnel_index, &[(".", true)])
            .await
        {
            log::error!("Failed to set search domains: {}", error.display_chain());
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, servers.to_vec())
            .await?;

        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        if let Err(error) = self
            .dbus_interface
            .set_domains(self.tunnel_index, &[])
            .await
        {
            log::error!("Failed to set search domains: {}", error.display_chain());
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, vec![])
            .await?;

        Ok(())
    }
}

impl Drop for SystemdResolved {
    fn drop(&mut self) {
        if let Some(backup) = self.backup.take()
            && let Err(error) = backup.restore()
        {
            trace!("{error}");
        }
    }
}

/// Backup of `/etc/resolv.conf`. Either it is a symlink or a file. In the latter case, store its
/// content and write back when we no longer manage DNS.
enum Backup {
    Symlink { original: Sensitive<PathBuf> },
    File { content: Sensitive<Box<[u8]>> },
}

impl Backup {
    /// Create a new backup from `/etc/resolv.conf`, if it is not already managed by
    /// `systemd-resolved`.
    fn new() -> Result<Option<Self>> {
        // Check that `/etc/resolv.conf` is a symlink to `/run/systemd/resolve/stub-resolv.conf`.
        if systemd_manages_resolv_conf().map_err(Error::ResolvConf)? {
            return Ok(None);
        }
        // Otherwise, back up the previous resolv.conf and restore the symlink.

        // Backup the existing `/etc/resolv.conf`
        let backup = Backup::create()?;
        // Re-link `/etc/resolv.conf` to `/run/systemd/resolve/stub-resolv.conf`.
        symlink("/run/systemd/resolve/stub-resolv.conf", "/etc/resolv.conf")?;
        Ok(Some(backup))
    }

    /// Read `/etc/resolv.conf` if it is a file, or stash the link if it is a symlink.
    fn create() -> io::Result<Backup> {
        let resolv_conf = Path::new("/etc/resolv.conf");
        if resolv_conf.is_symlink() {
            // `/etc/resolv.conf` is a file.
            let original = read_link(resolv_conf)?.into();
            return Ok(Backup::Symlink { original });
        }
        // `/etc/resolv.conf` is a file. Read its content.
        let content = fs::read(resolv_conf)?.into_boxed_slice().into();
        Ok(Backup::File { content })
    }

    fn restore(self) -> io::Result<()> {
        trace!("Restoring backup {self:?}");
        match self {
            Backup::Symlink { original } => {
                fs::remove_file("/etc/resolv.conf")?;
                symlink(original.as_path(), "/etc/resolv.conf")
            }
            Backup::File { content } => fs::write("/etc/resolv.conf", *content.as_ref()),
        }
    }
}

impl fmt::Debug for Backup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symlink { original } => f
                .debug_struct("Symlink")
                .field("original", original)
                .finish(),
            Self::File { content: _ } => f.debug_struct("File").finish(),
        }
    }
}

// Check if `/etc/resolv.conf` is a symlink to `/run/systemd/resolve/stub-resolv.conf`.
fn systemd_manages_resolv_conf() -> io::Result<bool> {
    let systemd_resolver_stub = Path::new("/etc/resolv.conf");
    if !systemd_resolver_stub.is_symlink() {
        warn!("/etc/resolve.conf is not a symlink");
        return Ok(false);
    }

    let link = canonicalize(systemd_resolver_stub)?;
    match link.to_string_lossy().as_ref() {
        "/run/systemd/resolve/stub-resolv.conf"
        | "/run/systemd/resolve/resolv.conf"
        | "/lib/systemd/resolv.conf" => {
            // all good
            Ok(true)
        }
        resolved => {
            trace!("resolve.conf points to {resolved}");
            warn!("resolve.conf is not a systemd-resolved target");
            Ok(false)
        }
    }
}
