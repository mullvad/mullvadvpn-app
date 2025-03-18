use anyhow::{anyhow, Context, Result};
use futures::future::{self, Either};
use nix::sys::socket::SockaddrStorage;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::{io::AsyncWriteExt, process::Command};

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
/// Port of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_REAL_PORT: u16 = 51820;
/// Tunnel address of the wireguard local peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_LOCAL_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 2);
/// Tunnel address of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 1);
/// Gateway (and default DNS resolver) of the wireguard tunnel.
pub const CUSTOM_TUN_GATEWAY: Ipv4Addr = CUSTOM_TUN_REMOTE_TUN_ADDR;
/// Name of the wireguard interface on the host
pub const CUSTOM_TUN_INTERFACE_NAME: &str = "utun123";

use std::time::Duration;

/// Timeout for wireguard-go to create an interface
const INTERFACE_SETUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Set up WireGuard relay and dummy hosts.
pub async fn setup_test_network() -> Result<()> {
    log::debug!("Setting up test network");

    enable_forwarding().await?;
    create_wireguard_interface()
        .await
        .context("Failed to create WireGuard interface")?;

    Ok(())
}

/// Returns the interface name and IP address of the bridge gateway, which is the (first) bridge
/// network that the given `guest_ip` belongs to.
pub(crate) fn find_vm_bridge(guest_ip: &Ipv4Addr) -> Result<(String, Ipv4Addr)> {
    let to_sock_addr = |addr: Option<SockaddrStorage>| {
        addr.as_ref()
            .and_then(|addr| addr.as_sockaddr_in())
            .map(|addr| *SocketAddrV4::from(*addr).ip())
    };

    nix::ifaddrs::getifaddrs()
        .unwrap()
        .filter(|addr| addr.interface_name.starts_with("bridge"))
        .filter_map(|addr| {
            let address = to_sock_addr(addr.address);
            let netmask = to_sock_addr(addr.netmask);
            address
                .zip(netmask)
                .map(|(address, netmask)| (addr.interface_name, address, netmask))
        })
        .find_map(|(interface_name, address, netmask)| {
            ipnetwork::Ipv4Network::with_netmask(address, netmask)
                .ok()
                .filter(|ip_v4_network| ip_v4_network.contains(*guest_ip))
                .map(|_| (interface_name.clone(), address))
        })
        .ok_or_else(|| anyhow!("Failed to identify bridge used by tart -- not running?"))
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

    let mut go_proc = tokio::spawn(async move {
        let mut cmd = Command::new("/usr/bin/sudo");
        cmd.kill_on_drop(true);
        cmd.args(["wireguard-go", "-f", CUSTOM_TUN_INTERFACE_NAME]);
        let output = cmd.output().await.context("Run wireguard-go")?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "wireguard-go failed with status {:?}",
                output.status.code()
            ))
        }
    });

    let mut tunnel_check: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        loop {
            // Check if the tunnel already exists
            let mut cmd = Command::new("/sbin/ifconfig");
            cmd.arg(CUSTOM_TUN_INTERFACE_NAME);
            let output = cmd
                .output()
                .await
                .context("Check if wireguard tunnel exists")?;
            if output.status.success() {
                log::debug!("Created custom WireGuard tunnel interface");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let result = tokio::time::timeout(
        INTERFACE_SETUP_TIMEOUT,
        future::select(&mut go_proc, &mut tunnel_check),
    )
    .await
    .context("WireGuard interface setup timed out")?;

    let result = match result {
        Either::Left((result, _)) | Either::Right((result, _)) => result,
    };

    tunnel_check.abort();

    result?
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
