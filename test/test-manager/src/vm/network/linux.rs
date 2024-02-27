use ipnetwork::Ipv4Network;
use once_cell::sync::Lazy;
use std::{
    ffi::OsStr,
    io,
    net::{IpAddr, Ipv4Addr},
    process::Stdio,
    str::FromStr,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
};

/// (Contained) test subnet for the test runner: 172.29.1.1/24
pub static TEST_SUBNET: Lazy<Ipv4Network> =
    Lazy::new(|| Ipv4Network::new(Ipv4Addr::new(172, 29, 1, 1), 24).unwrap());
/// Range of IPs returned by the DNS server: TEST_SUBNET_DHCP_FIRST to TEST_SUBNET_DHCP_LAST
pub const TEST_SUBNET_DHCP_FIRST: Ipv4Addr = Ipv4Addr::new(172, 29, 1, 2);
/// Range of IPs returned by the DNS server: TEST_SUBNET_DHCP_FIRST to TEST_SUBNET_DHCP_LAST
pub const TEST_SUBNET_DHCP_LAST: Ipv4Addr = Ipv4Addr::new(172, 29, 1, 128);

/// Bridge interface on the host
pub const BRIDGE_NAME: &str = "br-mullvadtest";
/// TAP interface used by the guest
pub const TAP_NAME: &str = "tap-mullvadtest";

/// Pingable dummy LAN interface (name)
pub const DUMMY_LAN_INTERFACE_NAME: &str = "lan-mullvadtest";
/// Pingable dummy LAN interface (IP)
pub const DUMMY_LAN_INTERFACE_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 1, 200);
/// Pingable dummy interface with public IP (name)
pub const DUMMY_INET_INTERFACE_NAME: &str = "net-mullvadtest";
/// Pingable dummy interface with public IP (IP)
pub const DUMMY_INET_INTERFACE_IP: Ipv4Addr = Ipv4Addr::new(1, 3, 3, 7);

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
#[allow(dead_code)]
pub const CUSTOM_TUN_REMOTE_REAL_ADDR: Ipv4Addr = Ipv4Addr::new(172, 29, 1, 200);
/// Port of the wireguard remote peer as defined in `setup-network.sh`.
#[allow(dead_code)]
pub const CUSTOM_TUN_REMOTE_REAL_PORT: u16 = 51820;
/// Tunnel address of the wireguard local peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_LOCAL_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 2);
/// Tunnel address of the wireguard remote peer as defined in `setup-network.sh`.
pub const CUSTOM_TUN_REMOTE_TUN_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 15, 1);
/// Gateway (and default DNS resolver) of the wireguard tunnel.
#[allow(dead_code)]
pub const CUSTOM_TUN_GATEWAY: Ipv4Addr = CUSTOM_TUN_REMOTE_TUN_ADDR;
/// Gateway of the non-tunnel interface.
#[allow(dead_code)]
pub const NON_TUN_GATEWAY: Ipv4Addr = Ipv4Addr::new(172, 29, 1, 1);
/// Name of the wireguard interface on the host
pub const CUSTOM_TUN_INTERFACE_NAME: &str = "wg-relay0";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to start 'ip'")]
    IpStart(io::Error),
    #[error("'ip' command failed: {0}")]
    IpFailed(i32),
    #[error("Failed to start 'sysctl'")]
    SysctlStart(io::Error),
    #[error("'sysctl' failed: {0}")]
    SysctlFailed(i32),
    #[error("Failed to start 'nft'")]
    NftStart(io::Error),
    #[error("Failed to wait for 'nft'")]
    NftRun(io::Error),
    #[error("'nft' command failed: {0}")]
    NftFailed(i32),
    #[error("Failed to create wg config")]
    CreateWireguardConfig(#[source] async_tempfile::Error),
    #[error("Failed to write wg config")]
    WriteWireguardConfig(#[source] io::Error),
    #[error("Failed to start 'wg'")]
    WgStart(io::Error),
    #[error("'wg' failed: {0}")]
    WgFailed(i32),
    #[error("Failed to start 'dnsmasq'")]
    DnsmasqStart(io::Error),
    #[error("Failed to create dnsmasq tempfile")]
    CreateDnsmasqFile(#[source] async_tempfile::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// TODO: probably provider dependent
pub struct NetworkHandle {
    dhcp_proc: DhcpProcHandle,
}

struct DhcpProcHandle {
    child: Child,
    _leases_file: async_tempfile::TempFile,
    _pid_file: async_tempfile::TempFile,
}

/// Create a bridge network and hosts
pub async fn setup_test_network() -> Result<NetworkHandle> {
    enable_forwarding().await?;

    let test_subnet = TEST_SUBNET.to_string();

    log::info!("Create bridge network: dev {BRIDGE_NAME}, net {test_subnet}");

    run_ip_cmd(["link", "add", BRIDGE_NAME, "type", "bridge"]).await?;
    run_ip_cmd(["addr", "add", "dev", BRIDGE_NAME, &test_subnet]).await?;
    run_ip_cmd(["link", "set", "dev", BRIDGE_NAME, "up"]).await?;

    log::debug!("Masquerade traffic from bridge to internet");

    run_nft(&format!(
        "
table ip mullvad_test_nat {{
    chain POSTROUTING {{
        type nat hook postrouting priority srcnat; policy accept;
        ip saddr {test_subnet} ip daddr != {test_subnet} counter masquerade
    }}
}}"
    ))
    .await?;

    log::debug!("Set up pingable hosts");

    run_ip_cmd(["link", "add", DUMMY_LAN_INTERFACE_NAME, "type", "dummy"]).await?;
    run_ip_cmd([
        "addr",
        "add",
        "dev",
        DUMMY_LAN_INTERFACE_NAME,
        &DUMMY_LAN_INTERFACE_IP.to_string(),
    ])
    .await?;

    run_ip_cmd(["link", "add", DUMMY_INET_INTERFACE_NAME, "type", "dummy"]).await?;
    run_ip_cmd([
        "addr",
        "add",
        "dev",
        DUMMY_INET_INTERFACE_NAME,
        &DUMMY_INET_INTERFACE_IP.to_string(),
    ])
    .await?;

    log::debug!("Create WireGuard peer");

    create_local_wireguard_peer().await?;

    log::debug!("Start DHCP server for {BRIDGE_NAME}");

    let dhcp_proc = start_dnsmasq().await?;

    log::debug!("Create TAP interface {TAP_NAME} for guest");

    run_ip_cmd(["tuntap", "add", TAP_NAME, "mode", "tap"]).await?;
    run_ip_cmd(["link", "set", TAP_NAME, "master", BRIDGE_NAME]).await?;
    run_ip_cmd(["link", "set", TAP_NAME, "up"]).await?;

    Ok(NetworkHandle { dhcp_proc })
}

impl NetworkHandle {
    /// Return the first IP address acknowledged by the DHCP server. This can only be called once.
    pub async fn first_dhcp_ack(&mut self) -> Option<IpAddr> {
        const LOG_PREFIX: &str = "[dnsmasq] ";
        const LOG_LEVEL: log::Level = log::Level::Debug;

        // dnsmasq-dhcp: DHCPACK(br-mullvadtest) 172.29.1.112 52:54:00:12:34:56 debian
        let re = regex::Regex::new(r"DHCPACK.*\) ([0-9.]+)").unwrap();

        let stderr = self.dhcp_proc.child.stderr.take();

        let reader = BufReader::new(stderr?);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            log::log!(LOG_LEVEL, "{LOG_PREFIX}{}", line);

            if let Some(addr) = re
                .captures(&line)
                .and_then(|cap| cap.get(1))
                .map(|addr| addr.as_str())
            {
                if let Ok(parsed_addr) = IpAddr::from_str(addr) {
                    log::debug!("Captured DHCPACK: {}", parsed_addr);
                    return Some(parsed_addr);
                }
            }
        }

        tokio::spawn(crate::vm::logging::forward_logs(
            LOG_PREFIX,
            lines.into_inner().into_inner(),
            LOG_LEVEL,
        ));

        None
    }
}

async fn start_dnsmasq() -> Result<DhcpProcHandle> {
    // dnsmasq -i BRIDGE_NAME -F TEST_SUBNET_DHCP_FIRST,TEST_SUBNET_DHCP_LAST ...
    let mut cmd = Command::new("dnsmasq");

    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd.args([
        "--bind-interfaces",
        "-C",
        "/dev/null",
        "-i",
        BRIDGE_NAME,
        "-F",
        &format!("{},{}", TEST_SUBNET_DHCP_FIRST, TEST_SUBNET_DHCP_LAST),
        "--no-daemon",
    ]);

    let leases_file = async_tempfile::TempFile::new()
        .await
        .map_err(Error::CreateDnsmasqFile)?;
    cmd.args(["-l", leases_file.file_path().to_str().unwrap()]);

    let pid_file = async_tempfile::TempFile::new()
        .await
        .map_err(Error::CreateDnsmasqFile)?;
    cmd.args(["-x", pid_file.file_path().to_str().unwrap()]);

    let child = cmd.spawn().map_err(Error::DnsmasqStart)?;

    Ok(DhcpProcHandle {
        child,
        _leases_file: leases_file,
        _pid_file: pid_file,
    })
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
async fn create_local_wireguard_peer() -> Result<()> {
    run_ip_cmd([
        "link",
        "add",
        "dev",
        CUSTOM_TUN_INTERFACE_NAME,
        "type",
        "wireguard",
    ])
    .await?;
    run_ip_cmd([
        "addr",
        "add",
        "dev",
        CUSTOM_TUN_INTERFACE_NAME,
        &CUSTOM_TUN_REMOTE_TUN_ADDR.to_string(),
        "peer",
        &CUSTOM_TUN_LOCAL_TUN_ADDR.to_string(),
    ])
    .await?;

    let mut tempfile = async_tempfile::TempFile::new()
        .await
        .map_err(Error::CreateWireguardConfig)?;

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
        .map_err(Error::WriteWireguardConfig)?;

    let mut cmd = Command::new("wg");
    cmd.args([
        "setconf",
        CUSTOM_TUN_INTERFACE_NAME,
        tempfile.file_path().to_str().unwrap(),
    ]);
    let output = cmd.output().await.map_err(Error::WgStart)?;
    if !output.status.success() {
        return Err(Error::WgFailed(output.status.code().unwrap()));
    }

    run_ip_cmd(["link", "set", "dev", CUSTOM_TUN_INTERFACE_NAME, "up"]).await?;

    Ok(())
}

async fn run_ip_cmd<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("ip");
    cmd.args(args);
    let output = cmd.output().await.map_err(Error::IpStart)?;
    if !output.status.success() {
        return Err(Error::IpFailed(output.status.code().unwrap()));
    }
    Ok(())
}

async fn run_nft(input: &str) -> Result<()> {
    let mut cmd = Command::new("nft");
    cmd.args(["-f", "-"]);

    cmd.stdin(Stdio::piped());

    let mut child = cmd.spawn().map_err(Error::NftStart)?;
    let mut stdin = child.stdin.take().unwrap();

    stdin
        .write_all(input.as_bytes())
        .await
        .expect("write to nft failed");

    drop(stdin);

    let output = child.wait_with_output().await.map_err(Error::NftRun)?;
    if !output.status.success() {
        return Err(Error::NftFailed(output.status.code().unwrap()));
    }
    Ok(())
}

async fn enable_forwarding() -> Result<()> {
    let mut cmd = Command::new("sysctl");
    cmd.arg("net.ipv4.ip_forward=1");
    let output = cmd.output().await.map_err(Error::SysctlStart)?;
    if !output.status.success() {
        return Err(Error::SysctlFailed(output.status.code().unwrap()));
    }
    Ok(())
}
