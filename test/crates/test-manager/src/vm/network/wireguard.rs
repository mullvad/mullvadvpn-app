//! Create and manage a local WireGuard interface.

use anyhow::{Context, Result};
use gotatun::device::{self, DefaultDeviceTransports, Device, Peer};
use gotatun::tun::tun_async_device::TunDevice;
use ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;

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
/// Name of the wireguard interface on the host
pub const CUSTOM_TUN_INTERFACE_NAME: &str = cfg_select! {
    target_os = "linux" => { "wg-relay0" }
    target_os = "macos" => { "utun123" }
};

/// Creates a WireGuard peer on the host.
///
/// This relay does not support PQ handshakes, etc.
///
/// The client should connect to `CUSTOM_TUN_REMOTE_REAL_ADDR` on port `CUSTOM_TUN_REMOTE_REAL_PORT`
/// using the private key `CUSTOM_TUN_LOCAL_PRIVKEY`, and tunnel IP `CUSTOM_TUN_LOCAL_TUN_ADDR`.
///
/// The public key of the peer is `CUSTOM_TUN_REMOTE_PUBKEY`. The tunnel IP of the host peer is
/// `CUSTOM_TUN_REMOTE_TUN_ADDR`.
pub(crate) async fn create_interface() -> Result<Device<DefaultDeviceTransports>> {
    log::debug!("Creating custom WireGuard tunnel");

    let peer = Peer::new(CUSTOM_TUN_LOCAL_PUBKEY.into()).with_allowed_ip(
        const { Ipv4Network::new_checked(CUSTOM_TUN_LOCAL_TUN_ADDR, 32).unwrap() }.into(),
    );

    let tun = {
        let tun = {
            let mut tun_config = tun::Configuration::default();
            tun_config.tun_name(CUSTOM_TUN_INTERFACE_NAME);
            tun_config.up();
            #[cfg(target_os = "macos")]
            tun_config.platform_config(|p| {
                p.enable_routing(false);
            });
            tun::create_as_async(&tun_config).context("Failed to open tun device")?
        };
        TunDevice::from_tun_device(tun).context("Failed to create utun device")?
    };

    let device = device::build()
        .with_default_udp()
        .with_ip(tun)
        .with_private_key(CUSTOM_TUN_REMOTE_PRIVKEY.into())
        .with_peer(peer)
        .with_listen_port(CUSTOM_TUN_REMOTE_REAL_PORT)
        .build()
        .await
        .context("Failed to create gotatun device")?;

    Ok(device)
}

#[cfg(target_os = "macos")]
pub async fn configure_tunnel() -> Result<()> {
    use anyhow::bail;
    use tokio::process::Command;
    // Check if the tunnel device is configured
    let mut cmd = Command::new("/usr/sbin/ipconfig");
    cmd.args(["getifaddr", CUSTOM_TUN_INTERFACE_NAME]);
    let output = cmd
        .output()
        .await
        .context("Check if wireguard tunnel has IP")?;
    if output.status.success() {
        log::debug!("Tunnel {CUSTOM_TUN_INTERFACE_NAME} already configured");
        return Ok(());
    }

    // Set tunnel IP address
    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args([
        "/usr/sbin/ipconfig",
        "set",
        CUSTOM_TUN_INTERFACE_NAME,
        "manual",
        &CUSTOM_TUN_REMOTE_TUN_ADDR.to_string(),
    ]);
    let status = cmd.status().await.context("Run ipconfig")?;
    if !status.success() {
        bail!("ipconfig failed: {}", status.code().unwrap());
    }
    Ok(())
}
