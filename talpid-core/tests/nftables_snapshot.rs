//! Snapshot tests of the actual nftables table set up for the various
//! [`FirewallPolicy`] states, and for transitions between them.
//!
//! Each test is run in throwaway user, network, cgroup and mount namespaces,
//! where the resulting `inet mullvad` table is dumped with `nft` and compared
//! against a snapshot. This lets the tests run rootless.
//!
//! The tests are ignored by default, since they depend on the environment. Run
//! them with `cargo test -p talpid-core --test nftables_snapshot -- --ignored`.
//! They fail on systems which cannot provide the namespaces, e.g. when
//! unprivileged user namespaces are disabled and the process is not privileged.

#![cfg(target_os = "linux")]

use std::{
    env,
    io::{self, Write as _},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    process::Command,
    sync::LazyLock,
};

use insta::assert_snapshot;
use regex_lite::Regex;
use talpid_core::firewall::{FirewallPolicy, linux::Firewall};
use talpid_dns::DnsConfig;
use talpid_tunnel::TunnelMetadata;
use talpid_types::net::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, TransportProtocol,
};

/// Env var marking the process as the (already isolated) re-executed test.
const INNER_ENV: &str = "TALPID_NFT_SNAPSHOT_ISOLATED";

const FWMARK: u32 = 0x6d6f_6c65;

/// The dummy interface standing in for the VPN tunnel device.
const TUNNEL_IFACE: &str = "mullvad-test0";

const RELAY_ENDPOINT: Endpoint = Endpoint::from_socket_address(
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(185, 65, 134, 1)), 51820),
    TransportProtocol::Udp,
);
const SECOND_RELAY_ENDPOINT: Endpoint = Endpoint::from_socket_address(
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(178, 162, 200, 100)), 51820),
    TransportProtocol::Udp,
);
const API_ENDPOINT: Endpoint = Endpoint::from_socket_address(
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(45, 83, 223, 198)), 443),
    TransportProtocol::Tcp,
);

const TUNNEL_IPV4: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 64, 11, 2));
const TUNNEL_IPV6: IpAddr = IpAddr::V6(Ipv6Addr::new(0xfc00, 0xbb, 0, 0, 0, 0, 0, 0x2));
const TUNNEL_IPV4_GATEWAY: Ipv4Addr = Ipv4Addr::new(10, 64, 11, 1);
const TUNNEL_IPV6_GATEWAY: Ipv6Addr = Ipv6Addr::new(0xfc00, 0xbb, 0, 0, 0, 0, 0, 0x1);

/// The `unshare` flags used to set up an isolated network namespace, or `None`
/// when the environment does not support isolation.
///
/// The unprivileged variant creates a user namespace and maps the current user
/// and group to root inside of it, which is what allows configuring the
/// network namespace. The root variant omits the user namespace and works in
/// privileged containers without user namespace support.
///
/// Both variants create cgroup and mount namespaces, so that a cgroup2
/// filesystem can be mounted without affecting the host. See [`mount_cgroup2`].
static ISOLATION_FLAGS: LazyLock<Option<&'static str>> = LazyLock::new(|| {
    for flags in ["-UrnCm", "-nCm"] {
        let probe = Command::new("unshare")
            .args([flags, "--", "/bin/true"])
            .output();
        if probe.is_ok_and(|output| output.status.success()) {
            return Some(flags);
        }
    }
    None
});

/// Runs `body` inside throwaway namespaces, as set up by [`ISOLATION_FLAGS`].
///
/// The body only runs in the re-executed test binary, in the fresh network
/// namespace. In the parent process this helper just waits for the child and
/// returns.
///
/// # Panics
///
/// Panics if the environment cannot provide the isolation, for example when
/// unprivileged user namespaces are disabled and the process is not privileged.
fn run_in_netns(test: &str, body: impl FnOnce()) {
    if env::var_os(INNER_ENV).is_some() {
        setup_netns_devices().expect("Failed to set up network namespace devices");
        body();
        return;
    }

    let flags = ISOLATION_FLAGS
        .expect("Failed to create isolated network namespaces (is util-linux installed?)");

    let executable = env::current_exe().expect("Failed to locate the current test executable");
    // Arguments meant for the test harness, minus the flags we need to set
    // ourselves, since the harness does not allow repeated options.
    let mut arguments: Vec<_> = env::args()
        .filter(|arg| !matches!(arg.as_str(), "--exact" | "--nocapture" | "--quiet"))
        .collect();
    arguments.push("--exact".into());
    arguments.push(test.into());
    arguments.push("--nocapture".into());
    arguments.push("--quiet".into());
    let status = Command::new("unshare")
        .arg(flags)
        .arg("--")
        .arg(executable)
        .args(&arguments)
        .env(INNER_ENV, "1")
        .status()
        .expect("Failed to run unshare, is the util-linux package installed?");

    match status.code() {
        Some(0) => (),
        Some(code) => {
            panic!("Test '{test}' failed in isolated network namespace (exit code {code})")
        }
        None => panic!("Test '{test}' was terminated in isolated network namespace"),
    }
}

/// Bring up loopback and create the tunnel stand-in interface.
fn setup_netns_devices() -> io::Result<()> {
    ip(&["link", "set", "lo", "up"])?;
    ip(&["link", "add", TUNNEL_IFACE, "type", "dummy"])
}

fn ip(arguments: &[&str]) -> io::Result<()> {
    let status = Command::new("ip").args(arguments).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("'ip {arguments:?}' failed")))
    }
}

/// Mount a cgroup2 filesystem at the default path, rooted at the cgroup
/// namespace. Must only be called from within [`run_in_netns`], since the mount
/// namespace is what keeps this from affecting the host.
///
/// This gives the test a root cgroup that it can write to, as long as the
/// cgroup that it runs in is owned by the current user.
fn mount_cgroup2() -> io::Result<()> {
    let status = Command::new("mount")
        .args(["-t", "cgroup2", "none", "/sys/fs/cgroup"])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("Failed to mount cgroup2"))
    }
}

/// Serializes the `mullvad` nftables table.
fn dump_nftables() -> String {
    let output = Command::new("nft")
        .args(["list", "table", "inet", "mullvad"])
        .output()
        .expect("Failed to run nft, is the nftables package installed?");
    if !output.status.success() {
        panic!(
            "Failed to list the mullvad nftables table: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    normalize(&String::from_utf8(output.stdout).expect("nft output is not UTF-8"))
}

/// Strip info that differs between environments.
fn normalize(dump: &str) -> String {
    // cgroup2 inode numbers
    let dump = Regex::new(r"(cgroupv2 level \d+) \d+")
        .unwrap()
        .replace_all(dump, "$1 <id>");
    // Older versions of nft print ICMPv6 code 0 by name
    dump.replace("icmpv6 code no-route", "icmpv6 code 0")
}

/// Firewall with a cleanup guard which removes the table on drop.
struct FirewallGuard {
    firewall: Firewall,
}

impl FirewallGuard {
    fn new(net_cls: Option<u32>) -> Self {
        FirewallGuard {
            firewall: Firewall::new(FWMARK, None, net_cls).expect("Failed to create firewall"),
        }
    }

    fn apply(&mut self, policy: FirewallPolicy) {
        self.firewall
            .apply_policy(policy)
            .expect("Failed to apply firewall policy");
    }
}

impl Drop for FirewallGuard {
    fn drop(&mut self) {
        let _ = self.firewall.reset_policy();
    }
}

fn tunnel_metadata() -> TunnelMetadata {
    TunnelMetadata {
        interface: TUNNEL_IFACE.to_string(),
        dummy_dns: None,
        ips: vec![TUNNEL_IPV4, TUNNEL_IPV6],
        ipv4_gateway: TUNNEL_IPV4_GATEWAY,
        ipv6_gateway: Some(TUNNEL_IPV6_GATEWAY),
    }
}

fn allowed_endpoint(endpoint: Endpoint) -> AllowedEndpoint {
    AllowedEndpoint {
        endpoint,
        clients: AllowedClients::Root,
    }
}

fn policy_blocked(allow_lan: bool, allowed_endpoint: Option<Endpoint>) -> FirewallPolicy {
    FirewallPolicy::Blocked {
        allow_lan,
        allowed_endpoint: allowed_endpoint.map(crate::allowed_endpoint),
    }
}

fn policy_connecting(
    tunnel: Option<TunnelMetadata>,
    allowed_tunnel_traffic: AllowedTunnelTraffic,
    allow_lan: bool,
) -> FirewallPolicy {
    FirewallPolicy::Connecting {
        peer_endpoints: vec![allowed_endpoint(RELAY_ENDPOINT)],
        tunnel,
        allow_lan,
        allowed_endpoint: allowed_endpoint(API_ENDPOINT),
        allowed_tunnel_traffic,
    }
}

fn policy_connected(allow_lan: bool, dns_config: DnsConfig) -> FirewallPolicy {
    FirewallPolicy::Connected {
        peer_endpoints: vec![allowed_endpoint(RELAY_ENDPOINT)],
        tunnel: tunnel_metadata(),
        allow_lan,
        dns_config: dns_config.resolve(&[]),
    }
}

fn tunnel_dns_config() -> DnsConfig {
    DnsConfig::from_addresses(
        &[IpAddr::V4(Ipv4Addr::new(10, 64, 0, 1))],
        &[IpAddr::V6(Ipv6Addr::new(0xfc00, 0xbb, 0, 0, 0, 0, 0, 0x1))],
    )
}

fn local_dns_config() -> DnsConfig {
    DnsConfig::from_addresses(
        &[IpAddr::V6(Ipv6Addr::new(0xfc00, 0xbb, 0, 0, 0, 0, 0, 0x1))],
        &[
            IpAddr::V4(Ipv4Addr::new(10, 64, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
        ],
    )
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn blocked() {
    run_in_netns("blocked", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_blocked(false, None));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn blocked_with_allowed_endpoint() {
    run_in_netns("blocked_with_allowed_endpoint", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_blocked(false, Some(API_ENDPOINT)));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn blocked_allow_lan() {
    run_in_netns("blocked_allow_lan", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_blocked(true, None));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connecting_without_tunnel() {
    run_in_netns("connecting_without_tunnel", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connecting(None, AllowedTunnelTraffic::All, false));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connecting_with_tunnel() {
    run_in_netns("connecting_with_tunnel", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connecting(
            Some(tunnel_metadata()),
            AllowedTunnelTraffic::All,
            false,
        ));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connecting_with_tunnel_allow_lan() {
    run_in_netns("connecting_with_tunnel_allow_lan", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connecting(
            Some(tunnel_metadata()),
            AllowedTunnelTraffic::All,
            true,
        ));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connecting_no_in_tunnel_traffic() {
    run_in_netns("connecting_no_in_tunnel_traffic", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connecting(
            Some(tunnel_metadata()),
            AllowedTunnelTraffic::None,
            false,
        ));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connecting_one_in_tunnel_endpoint() {
    run_in_netns("connecting_one_in_tunnel_endpoint", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connecting(
            Some(tunnel_metadata()),
            AllowedTunnelTraffic::One(SECOND_RELAY_ENDPOINT),
            false,
        ));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connecting_two_in_tunnel_endpoints() {
    run_in_netns("connecting_two_in_tunnel_endpoints", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connecting(
            Some(tunnel_metadata()),
            AllowedTunnelTraffic::Two(RELAY_ENDPOINT, SECOND_RELAY_ENDPOINT),
            false,
        ));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connected() {
    run_in_netns("connected", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connected(false, tunnel_dns_config()));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connected_allow_lan() {
    run_in_netns("connected_allow_lan", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connected(true, tunnel_dns_config()));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connected_local_dns() {
    run_in_netns("connected_local_dns", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connected(false, local_dns_config()));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connected_with_split_tunnel() {
    run_in_netns("connected_with_split_tunnel", || {
        let mut firewall = FirewallGuard::new(Some(4610));
        firewall.apply(policy_connected(false, tunnel_dns_config()));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn connected_with_cgroup2_split_tunnel() {
    // Only meaningful when built with `--features cgroup2`, and it is skipped
    // otherwise. It fails unless it runs in a cgroup owned by the current user.
    run_in_netns("connected_with_cgroup2_split_tunnel", || {
        if !cfg!(feature = "cgroup2") {
            let _ = writeln!(
                io::stderr(),
                "skipping test: talpid-core not built with the cgroup2 feature"
            );
            return;
        }
        // Containers usually have a usable cgroup2 mounted already, in which
        // case mounting the same cgroup on top of it fails.
        let cgroup2 = talpid_cgroup::v2::CGroup2::open_root().unwrap_or_else(|_| {
            mount_cgroup2().expect("Failed to mount cgroup2 filesystem");
            talpid_cgroup::v2::CGroup2::open_root().expect("Failed to open root cgroup2")
        });
        let mut firewall =
            Firewall::new(FWMARK, Some(cgroup2), None).expect("Failed to create firewall");
        firewall
            .apply_policy(policy_connected(false, tunnel_dns_config()))
            .expect("Failed to apply firewall policy");
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn disconnecting() {
    run_in_netns("disconnecting", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(FirewallPolicy::Disconnecting { allow_lan: false });
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn transition_blocked_to_connected() {
    run_in_netns("transition_blocked_to_connected", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_blocked(false, Some(API_ENDPOINT)));
        firewall.apply(policy_connecting(
            Some(tunnel_metadata()),
            AllowedTunnelTraffic::All,
            false,
        ));
        firewall.apply(policy_connected(false, tunnel_dns_config()));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn transition_connected_to_blocked() {
    run_in_netns("transition_connected_to_blocked", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_connected(true, tunnel_dns_config()));
        firewall.apply(policy_blocked(true, None));
        assert_snapshot!(dump_nftables());
    });
}

#[test]
#[ignore = "requires network namespaces, ip and nft"]
fn reset_removes_table() {
    run_in_netns("reset_removes_table", || {
        let mut firewall = FirewallGuard::new(None);
        firewall.apply(policy_blocked(false, None));
        firewall
            .firewall
            .reset_policy()
            .expect("Failed to reset firewall policy");

        let output = Command::new("nft")
            .args(["list", "ruleset"])
            .output()
            .expect("Failed to run nft");
        if !output.status.success() {
            panic!(
                "Failed to list the nftables ruleset: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert_snapshot!(String::from_utf8(output.stdout).expect("nft output is not UTF-8"));
    });
}
