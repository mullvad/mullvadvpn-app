use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{anyhow, Context, Result};
use tokio::{io::AsyncWriteExt, process::Command};

/// Pingable dummy LAN interface (IP)
/// TODO: This should probably be a different host, not the gateway
pub const DUMMY_LAN_INTERFACE_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 64, 1);

// Private key of the wireguard remote peer on host.
const CUSTOM_TUN_REMOTE_PRIVKEY: &str = "gLvQuyqazziyf+pUCAFUgTnWIwn6fPE5MOReOqPEGHU=";
// Public key of the wireguard remote peer on host.
data_encoding_macro::base64_array!(
    "pub const CUSTOM_TUN_REMOTE_PUBKEY" = "7svBwGBefP7KVmH/yes+pZCfO6uSOYeGieYYa1+kZ0E="
);
// Private key of the wireguard local peer on guest.
const CUSTOM_TUN_LOCAL_PUBKEY: &str = "h6elqt3dfamtS/p9jxJ8bIYs8UW9YHfTFhvx0fabTFo=";
// Private key of the wireguard local peer on guest.
data_encoding_macro::base64_array!(
    "pub const CUSTOM_TUN_LOCAL_PRIVKEY" = "mPue6Xt0pdz4NRAhfQSp/SLKo7kV7DW+2zvBq0N9iUI="
);
/// "Real" (non-tunnel) IP of the wireguard remote peer as defined in `setup-network.sh`.
/// TODO: This should not be hardcoded. Set by tart.
pub const CUSTOM_TUN_REMOTE_REAL_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 64, 1);
/// Port of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_REAL_PORT: u16 = 51820;
/// Tunnel address of the wireguard local peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_LOCAL_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 2);
/// Tunnel address of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 1);
/// Gateway (and default DNS resolver) of the wireguard tunnel.
pub const CUSTOM_TUN_GATEWAY: Ipv4Addr = CUSTOM_TUN_REMOTE_TUN_ADDR;
/// Gateway of the non-tunnel interface.
/// TODO: This should not be hardcoded. Set by tart.
pub const NON_TUN_GATEWAY: Ipv4Addr = Ipv4Addr::new(192, 168, 64, 1);
/// Name of the wireguard interface on the host
pub const CUSTOM_TUN_INTERFACE_NAME: &str = "utun123";

/// Set up WireGuard relay and dummy hosts.
pub async fn setup_test_network() -> Result<()> {
    log::debug!("Setting up test network");

    enable_forwarding().await?;
    create_wireguard_interface()
        .await
        .context("Failed to create WireGuard interface")?;

    Ok(())
}

/// A hack to find the Tart bridge interface using `NON_TUN_GATEWAY`.
/// It should be possible to retrieve this using the virtualization framework instead,
/// but that requires an entitlement.
pub fn find_vm_bridge() -> Result<String> {
    for addr in nix::ifaddrs::getifaddrs().unwrap() {
        if !addr.interface_name.starts_with("bridge") {
            continue;
        }
        if let Some(address) = addr.address.as_ref().and_then(|addr| addr.as_sockaddr_in()) {
            let interface_ip = *SocketAddrV4::from(*address).ip();
            if interface_ip == NON_TUN_GATEWAY {
                return Ok(addr.interface_name.to_owned());
            }
        }
    }

    // This is probably either due to IP mismatch or Tart not running
    Err(anyhow!(
        "Failed to identify bridge used by tart -- not running?"
    ))
}

async fn enable_forwarding() -> Result<()> {
    // Enable forwarding
    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args(["/usr/sbin/sysctl", "net.inet.ip.forwarding=1"]);
    let output = cmd.output().await.context("Run sysctl")?;
    if !output.status.success() {
        return Err(anyhow!("sysctl failed: {}", output.status.code().unwrap()));
    }
    Ok(())
}

async fn create_wireguard_interface() -> Result<()> {
    log::debug!("Creating custom WireGuard tunnel");

    // Check if the tunnel already exists
    let mut cmd = Command::new("/sbin/ifconfig");
    cmd.arg(CUSTOM_TUN_INTERFACE_NAME);
    let output = cmd
        .output()
        .await
        .context("Check if wireguard tunnel exists")?;
    if output.status.success() {
        log::debug!("Tunnel {CUSTOM_TUN_INTERFACE_NAME} already exists");
    } else {
        let mut cmd = Command::new("/usr/bin/sudo");
        cmd.args(["wireguard-go", CUSTOM_TUN_INTERFACE_NAME]);
        let output = cmd.output().await.context("Run wireguard-go")?;
        if !output.status.success() {
            return Err(anyhow!(
                "wireguard-go failed: {}",
                output.status.code().unwrap()
            ));
        }
    }

    Ok(())
}

pub async fn configure_tunnel() -> Result<()> {
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

    // Set wireguard config
    let mut tempfile = async_tempfile::TempFile::new()
        .await
        .context("Failed to create temporary wireguard config")?;

    tempfile
        .write_all(
            format!(
                "

[Interface]
PrivateKey = {CUSTOM_TUN_REMOTE_PRIVKEY}
ListenPort = {CUSTOM_TUN_REMOTE_REAL_PORT}

[Peer]
PublicKey = {CUSTOM_TUN_LOCAL_PUBKEY}
AllowedIPs = {CUSTOM_TUN_LOCAL_TUN_ADDR}

"
            )
            .as_bytes(),
        )
        .await
        .context("Failed to write wireguard config")?;

    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args([
        "wg",
        "setconf",
        CUSTOM_TUN_INTERFACE_NAME,
        tempfile.file_path().to_str().unwrap(),
    ]);
    let output = cmd.output().await.context("Run wg")?;
    if !output.status.success() {
        return Err(anyhow!("wg failed: {}", output.status.code().unwrap()));
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
        return Err(anyhow!("ipconfig failed: {}", status.code().unwrap()));
    }
    Ok(())
}
