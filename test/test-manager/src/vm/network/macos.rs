use anyhow::{Context, Result, anyhow, bail};
use futures::{FutureExt, TryFutureExt, select};
use nix::{
    sys::{
        signal::{Signal, kill},
        socket::SockaddrStorage,
    },
    unistd::Pid,
};
use std::{
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
};
use talpid_types::drop_guard::on_drop;
use tokio::{io::AsyncWriteExt, process::Command, time::sleep};

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
/// Port of the wireguard remote peer
pub const CUSTOM_TUN_REMOTE_REAL_PORT: u16 = 51820;
/// Tunnel address of the wireguard local peer
pub const CUSTOM_TUN_LOCAL_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 2);
/// Tunnel address of the wireguard remote peer
pub const CUSTOM_TUN_REMOTE_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 1);
/// Subnet mask of the wireguard remote peer
pub const CUSTOM_TUN_REMOTE_TUN_SUBNET: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 255);
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

    {
        let mut r = std::process::Command::new("netstat");
        r.arg("-rn");
        let out = r.output();
        log::error!("netstat output: {:?}", out);
    }

    enable_forwarding().await?;
    create_wireguard_interface()
        .await
        .context("Failed to create WireGuard interface")?;

    log::info!("ran create_wireguard_interface");
    {
        let mut r = std::process::Command::new("netstat");
        r.arg("-rn");
        let out = r.output();
        log::error!("netstat output: {:?}", out);
    }

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

    // Create a future that spawns wireguard-go, and SIGTERMs it when dropped.
    let wireguard_go = async move {
        let mut cmd = Command::new("/usr/bin/sudo");
        cmd.args(["wireguard-go", "-f", CUSTOM_TUN_INTERFACE_NAME]);

        // We don't want to SIGKILL sudo, as that would leave wireguard-go orphaned and running
        cmd.kill_on_drop(false);
        let child = cmd.spawn().context("Failed to spawn wireguard-go")?;

        let pid = child.id().context("wireguard-go exited prematurely")?;
        let pid = Pid::from_raw(pid as libc::pid_t);

        let _term_on_drop = on_drop(|| {
            if let Err(e) = kill(pid, Signal::SIGTERM) {
                log::warn!("Failed to kill wireguard-go ({pid}): {e}");
            }
        });

        let output = child.wait_with_output().await.context("Run wireguard-go")?;
        if output.status.success() {
            bail!("wireguard-go exited prematurely")
        } else {
            bail!("wireguard-go failed with status {:?}", output.status.code())
        }
    };

    // Spawn wireguard-go using a tokio task. The task will hang around until this process exits.
    let wireguard_go = tokio::spawn(wireguard_go.inspect_err(|e| log::error!("{e}")));

    // Create a future that waits until the tunnel interface appears.
    let tunnel_check = async move {
        loop {
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
            sleep(Duration::from_secs(1)).await;
        }
    };

    // Wait until...
    select! {
        // ...the utun-interface is created
        result = tunnel_check.fuse() => result,

        // ...or wireguard-go exits with an error
        result = wireguard_go.fuse() => result
            .context("wireguard-go task panicked")?
            .map(|never: Infallible| match never {}), // this task never exits with Ok

        // ...or we hit the timeout
        _timeout = sleep(INTERFACE_SETUP_TIMEOUT).fuse() => {
            bail!("WireGuard interface setup timed out");
        }
    }
}

pub async fn configure_tunnel(ip_addr: IpAddr) -> Result<()> {
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

    // FIXME: I think some 192.168.* routes are being removed
    log::info!("pre setconf");
    {
        let mut r = std::process::Command::new("netstat");
        r.arg("-rn");
        let out = r.output();
        log::error!("netstat output: {:?}", out);
    }

    {
        let mut r = std::process::Command::new("ping");
        r.args(&["-c".to_string(), "1".to_string(), ip_addr.to_string()]);
        let out = r.output();
        log::error!("ping output: {:?}", out);
    }

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

    log::info!("ran setconf");
    {
        let mut r = std::process::Command::new("netstat");
        r.arg("-rn");
        let out = r.output();
        log::error!("netstat output: {:?}", out);
    }
    {
        let mut r = std::process::Command::new("ping");
        r.args(&["-c".to_string(), "1".to_string(), ip_addr.to_string()]);
        let out = r.output();
        log::error!("ping output: {:?}", out);
    }

    // Set tunnel IP address
    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args([
        "/usr/sbin/ipconfig",
        "set",
        CUSTOM_TUN_INTERFACE_NAME,
        "manual",
        &CUSTOM_TUN_REMOTE_TUN_ADDR.to_string(),
        &CUSTOM_TUN_REMOTE_TUN_SUBNET.to_string(),
    ]);
    let status = cmd.status().await.context("Run ipconfig")?;
    if !status.success() {
        return Err(anyhow!("ipconfig failed: {}", status.code().unwrap()));
    }
    log::info!("ran ipconfig");
    {
        let mut r = std::process::Command::new("netstat");
        r.arg("-rn");
        let out = r.output();
        log::error!("netstat output: {:?}", out);
    }
    {
        let mut r = std::process::Command::new("ping");
        r.args(&["-c".to_string(), "1".to_string(), ip_addr.to_string()]);
        let out = r.output();
        log::error!("ping output: {:?}", out);
    }
    Ok(())
}
