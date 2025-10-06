use ipnetwork::{Ipv4Network, Ipv6Network};
use std::{
    ffi::OsStr,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    process::Stdio,
    str::FromStr,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
};

/// (Contained) IPv4 subnet for the test runner: 172.29.1.1/24
pub const TEST_SUBNET_IPV4: Ipv4Network =
    Ipv4Network::new_checked(Ipv4Addr::new(172, 29, 1, 1), 24).unwrap();

/// IPv4 range returned by the DHCP server.
pub const TEST_SUBNET_IPV4_DHCP: RangeInclusive<Ipv4Addr> =
    Ipv4Addr::new(172, 29, 1, 2)..=Ipv4Addr::new(172, 29, 1, 128);

/// IPv6 subnet for the test runner. "0xfd multest"
pub const TEST_SUBNET_IPV6: Ipv6Network = Ipv6Network::new_checked(
    Ipv6Addr::new(0xfd6d, 0x756c, 0x7465, 0x7374, 0, 0, 0, 1),
    64,
)
.unwrap();

/// Bridge interface on the host
pub(crate) const BRIDGE_NAME: &str = "br-mullvadtest";
/// TAP interface used by the guest
pub const TAP_NAME: &str = "tap-mullvadtest";

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
pub(super) const NON_TUN_GATEWAY: Ipv4Addr = Ipv4Addr::new(172, 29, 1, 1);
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

/// IPv6-support in `rootlesskit` is experimental, and addresses are not automatically assigned.
/// This function will assigned an IPv6 address to the TAP interface, and set up routes.
async fn fix_ipv6() -> Result<()> {
    let tap = "tap0"; // TAP-device that connects to slirp2netns
    let addr = "fd00::1337/64"; // our address within the slirp2netns subnet
    let gateway = "fd00::2"; // slirp2netns gateway
    let _dns = "fd00::3"; // slirp2netns dns

    run_ip_cmd(["-6", "addr", "add", addr, "dev", tap]).await?;
    run_ip_cmd(["-6", "route", "add", "default", "via", gateway, "dev", tap]).await?;
    Ok(())
}

/// Create a bridge network and hosts
pub async fn setup_test_network() -> Result<NetworkHandle> {
    fix_ipv6().await?;

    enable_forwarding().await?;

    let test_subnet_v4 = TEST_SUBNET_IPV4.to_string();
    let test_subnet_v6 = TEST_SUBNET_IPV6.to_string();

    log::debug!("Create bridge network: dev {BRIDGE_NAME}, net {test_subnet_v4}");

    run_ip_cmd(["link", "add", BRIDGE_NAME, "type", "bridge"]).await?;
    run_ip_cmd(["addr", "add", "dev", BRIDGE_NAME, &test_subnet_v4]).await?;
    run_ip_cmd(["addr", "add", "dev", BRIDGE_NAME, &test_subnet_v6]).await?;
    run_ip_cmd(["link", "set", "dev", BRIDGE_NAME, "up"]).await?;

    log::debug!("Masquerade traffic from bridge to internet");

    run_nft(&format!(
        "
table inet mullvad_test_nat {{
    chain POSTROUTING {{
        type nat hook postrouting priority srcnat; policy accept;
        ip  saddr {test_subnet_v4} ip  daddr != {test_subnet_v4} counter masquerade
        ip6 saddr {test_subnet_v6} ip6 daddr != {test_subnet_v6} counter masquerade
    }}
}}"
    ))
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
        const LOG_PREFIX_STDOUT: &str = "[dnsmasq] [stdout] ";
        const LOG_PREFIX_STDERR: &str = "[dnsmasq] [stderr] ";
        const LOG_LEVEL: log::Level = log::Level::Debug;

        // dnsmasq-dhcp: DHCPACK(br-mullvadtest) 172.29.1.112 52:54:00:12:34:56 debian
        let re = regex::Regex::new(r"DHCPACK.*\) ([0-9.]+)").unwrap();

        let stderr = self.dhcp_proc.child.stderr.take();

        let reader = BufReader::new(stderr?);
        let mut lines = reader.lines();

        let mut found_addr = None;

        while let Ok(Some(line)) = lines.next_line().await {
            log::log!(LOG_LEVEL, "{LOG_PREFIX_STDERR}{}", line);

            if let Some(addr) = re
                .captures(&line)
                .and_then(|cap| cap.get(1))
                .map(|addr| addr.as_str())
            {
                if let Ok(parsed_addr) = IpAddr::from_str(addr) {
                    log::debug!("Captured DHCPACK: {}", parsed_addr);
                    found_addr = Some(parsed_addr);
                    break;
                }
            }
        }

        if let Some(stdout) = self.dhcp_proc.child.stdout.take() {
            tokio::spawn(crate::vm::logging::forward_logs(
                LOG_PREFIX_STDOUT,
                stdout,
                LOG_LEVEL,
            ));
        }
        tokio::spawn(crate::vm::logging::forward_logs(
            LOG_PREFIX_STDERR,
            lines.into_inner().into_inner(),
            LOG_LEVEL,
        ));

        found_addr
    }
}

/// Run dnsmasq as a DHCP server.
///
/// dnsmasq will serve IPv4 addresses within the range [TEST_SUBNET_IPV4_DHCP] using regular DHCP.
/// It will also advertise SLAAC for IPv6 within [TEST_SUBNET_IPV6].
async fn start_dnsmasq() -> Result<DhcpProcHandle> {
    let mut cmd = Command::new("dnsmasq");

    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd.args([
        "--conf-file=/dev/null",
        "--bind-interfaces",
        &format!("--interface={BRIDGE_NAME}"),
        // IPv4
        &format!(
            "--dhcp-range={},{}",
            TEST_SUBNET_IPV4_DHCP.start(),
            TEST_SUBNET_IPV4_DHCP.end(),
        ),
        // IPv6
        &format!(
            "--dhcp-range={prefix},slaac,{prefix_len}",
            prefix = TEST_SUBNET_IPV6.ip(),
            prefix_len = TEST_SUBNET_IPV6.prefix()
        ),
        "--no-hosts",
        "--keep-in-foreground",
        "--log-facility=-",
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

pub async fn run_nft(input: &str) -> Result<()> {
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
    let sysctl = "/usr/sbin/sysctl";
    let run = async |cmd: &mut Command| {
        let exit_status = cmd.output().await.map_err(Error::SysctlStart)?.status;
        match exit_status.success() {
            true => Ok(()),
            false => Err(Error::SysctlFailed(exit_status.code().unwrap())),
        }
    };
    run(Command::new(sysctl).arg("net.ipv4.ip_forward=1")).await?;
    run(Command::new(sysctl).arg("net.ipv6.conf.all.forwarding=1")).await?;
    Ok(())
}
