//! Create and manage a local WireGuard interface.

use anyhow::{Context, Result};
use gotatun::device::{self, DefaultDeviceTransports, Device, Peer};
use gotatun::tun::tun_async_device::TunDevice;
use ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;
use std::sync::RwLock;

// Private key of the wireguard remote peer on host.
data_encoding_macro::base64_array!(
    "pub const CUSTOM_TUN_REMOTE_PRIVKEY" = "gLvQuyqazziyf+pUCAFUgTnWIwn6fPE5MOReOqPEGHU="
);
// Public key of the wireguard remote peer on host.
data_encoding_macro::base64_array!(
    "pub const CUSTOM_TUN_REMOTE_PUBKEY" = "7svBwGBefP7KVmH/yes+pZCfO6uSOYeGieYYa1+kZ0E="
);
// Private key of the wireguard local peer on guest.
data_encoding_macro::base64_array!(
    "pub const CUSTOM_TUN_LOCAL_PUBKEY" = "h6elqt3dfamtS/p9jxJ8bIYs8UW9YHfTFhvx0fabTFo="
);
// Private key of the wireguard local peer on guest.
data_encoding_macro::base64_array!(
    "pub const CUSTOM_TUN_LOCAL_PRIVKEY" = "mPue6Xt0pdz4NRAhfQSp/SLKo7kV7DW+2zvBq0N9iUI="
);

/// Port of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_REAL_PORT: u16 = 51820;
/// Tunnel address of the wireguard local peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_LOCAL_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 2);
/// Tunnel address of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 1);
/// Gateway (and default DNS resolver) of the wireguard tunnel.
pub const CUSTOM_TUN_GATEWAY: Ipv4Addr = CUSTOM_TUN_REMOTE_TUN_ADDR;
/// Preferred name of the wireguard interface on the host
pub const CUSTOM_TUN_INTERFACE_NAME: &str = cfg_select! {
    target_os = "linux" => { "wg-relay0" }
    target_os = "macos" => { "utun123" }
};

/// The actual name of the wireguard interface on the host, set when the interface is created.
///
/// On macOS, the preferred name may be taken by another process, in which case a nearby free
/// interface name is used instead.
static TUN_INTERFACE_NAME: RwLock<Option<String>> = RwLock::new(None);

/// Returns the name of the wireguard interface on the host.
///
/// # Panics
///
/// If the interface has not been created yet.
pub fn tun_interface_name() -> String {
    TUN_INTERFACE_NAME
        .read()
        .unwrap()
        .clone()
        .expect("The wireguard interface has not been created")
}

/// Creates a WireGuard peer on the host.
///
/// This relay does not support PQ handshakes, etc.
///
/// The client should connect to `CUSTOM_TUN_REMOTE_REAL_ADDR` on port `CUSTOM_TUN_REMOTE_REAL_PORT`
/// using the private key `CUSTOM_TUN_LOCAL_PRIVKEY`, and tunnel IP `CUSTOM_TUN_LOCAL_TUN_ADDR`.
///
/// The public key of the peer is `CUSTOM_TUN_REMOTE_PUBKEY`. The tunnel IP of the host peer is
/// `CUSTOM_TUN_REMOTE_TUN_ADDR`.
///
/// Returns the device along with the name of its interface.
pub(crate) async fn create_interface() -> Result<(Device<DefaultDeviceTransports>, String)> {
    log::debug!("Creating custom WireGuard tunnel");

    let peer = Peer::new(CUSTOM_TUN_LOCAL_PUBKEY.into()).with_allowed_ip(
        const { Ipv4Network::new_checked(CUSTOM_TUN_LOCAL_TUN_ADDR, 32).unwrap() }.into(),
    );

    let tun = create_tun_device().context("Failed to create tun device")?;
    let interface_name = tun.name().context("Failed to get tun device name")?;
    *TUN_INTERFACE_NAME.write().unwrap() = Some(interface_name.clone());

    let device = device::build()
        .with_default_udp()
        .with_ip(tun)
        .with_private_key(CUSTOM_TUN_REMOTE_PRIVKEY.into())
        .with_peer(peer)
        .with_listen_port(CUSTOM_TUN_REMOTE_REAL_PORT)
        .build()
        .await
        .context("Failed to create gotatun device")?;

    Ok((device, interface_name))
}

/// Create the tun device backing the wireguard interface.
///
/// On macOS, utun interface units are a global resource. If the preferred unit is already taken,
/// e.g. by a previous test run that has not fully cleaned up, try neighboring units until a free
/// one is found.
fn create_tun_device() -> Result<TunDevice> {
    #[cfg(target_os = "macos")]
    {
        /// Number of neighboring interface units to try when the preferred one is taken.
        const FALLBACK_UNITS: u32 = 20;

        let preferred_unit: u32 = CUSTOM_TUN_INTERFACE_NAME
            .strip_prefix("utun")
            .and_then(|unit| unit.parse().ok())
            .expect("The custom tun interface must be named utunN");

        let mut last_error = None;
        for unit in preferred_unit..preferred_unit + FALLBACK_UNITS {
            let name = format!("utun{unit}");
            match create_tun_with_name(&name) {
                Ok(tun) => return Ok(tun),
                Err(error) => {
                    log::debug!("Failed to create {name}: {error:#}");
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.expect("At least one interface name was attempted"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        create_tun_with_name(CUSTOM_TUN_INTERFACE_NAME)
    }
}

fn create_tun_with_name(name: &str) -> Result<TunDevice> {
    let tun = {
        let mut tun_config = tun::Configuration::default();
        tun_config.tun_name(name);
        tun_config.up();
        #[cfg(target_os = "macos")]
        tun_config.platform_config(|p| {
            p.enable_routing(false);
        });
        tun::create_as_async(&tun_config).with_context(|| format!("Failed to open {name}"))?
    };
    TunDevice::from_tun_device(tun).context("Failed to create tun device")
}

#[cfg(target_os = "macos")]
pub async fn configure_tunnel(interface_name: &str) -> Result<()> {
    use anyhow::bail;
    use tokio::process::Command;
    // Check if the tunnel device is configured
    let mut cmd = Command::new("/usr/sbin/ipconfig");
    cmd.args(["getifaddr", interface_name]);
    let output = cmd
        .output()
        .await
        .context("Check if wireguard tunnel has IP")?;
    if output.status.success() {
        log::debug!("Tunnel {interface_name} already configured");
        return Ok(());
    }

    // Set tunnel IP address
    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args([
        "/usr/sbin/ipconfig",
        "set",
        interface_name,
        "manual",
        &CUSTOM_TUN_REMOTE_TUN_ADDR.to_string(),
    ]);
    let status = cmd.status().await.context("Run ipconfig")?;
    if !status.success() {
        bail!("ipconfig failed: {}", status.code().unwrap());
    }
    Ok(())
}
