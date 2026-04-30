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
pub const CUSTOM_TUN_INTERFACE_NAME: &str = "wg-relay0";

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

    let tun =
        TunDevice::from_name(CUSTOM_TUN_INTERFACE_NAME).context("Failed to create utun device")?;

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
