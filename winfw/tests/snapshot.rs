#![cfg(target_os = "windows")]

//! Snapshot tests for every WinFW firewall state.
//!
//! Each test drives the firewall implementation through one or more states,
//! dumping the complete WFP object state after each step and comparing it with
//! the snapshots in `tests/snapshots/`.
//!
//! The tests are ignored by default: they replace the WFP objects of any
//! running Mullvad daemon, and must run as administrator. Run them with
//! `cargo test -p winfw -- --ignored`. Regenerate snapshots with
//! `INSTA_UPDATE=always cargo test -p winfw -- --ignored`, or use
//! `cargo insta test -p winfw -- --ignored` and `cargo insta review` for the
//! review workflow.

mod common;

use std::sync::{Mutex, OnceLock};

use common::{Implementation, assert_snapshot, wfp_state};
use winfw::cpp_ffi::{
    AllowedTunnelTraffic, EndpointArgument, WinFwActivePolicy, WinFwCleanupPolicy,
    WinFwPolicyStatus, WinFwProtocol, WinFwSettings,
};

/// Timeout for acquiring the WFP transaction lock, in seconds. Same as the daemon.
pub const TIMEOUT_SECONDS: u32 = 5;

/// Client processes that exist on every Windows installation, used to exercise
/// the per-application firewall rules. The full paths are passed to the firewall,
/// which resolves them to WFP app identifiers.
const CLIENT_SVCHOST: &str = r"C:\Windows\System32\svchost.exe";
const CLIENT_CMD: &str = r"C:\Windows\System32\cmd.exe";

/// The alias of an existing interface, resolved by the harness.
fn tunnel_alias() -> String {
    common::tunnel_alias()
}

/// Serialize all tests: the firewall library is a per-process singleton and
/// the WFP state it holds is global.
fn lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// A driver that initializes the firewall on creation and deinitializes it
/// (with a full reset) when dropped, even if a test panics.
///
/// It holds the global lock for its whole lifetime: the firewall library is a
/// per-process singleton and the WFP state it holds is global.
struct Firewall {
    fw: Implementation,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Firewall {
    fn new() -> Self {
        // The guard is leaked from the `OnceLock` static mutex, so it outlives
        // this function call.
        let guard = lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut fw = Implementation::load();
        fw.initialize();
        Self { fw, _guard: guard }
    }
}

impl std::ops::Deref for Firewall {
    type Target = Implementation;

    fn deref(&self) -> &Self::Target {
        &self.fw
    }
}

impl std::ops::DerefMut for Firewall {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fw
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        self.fw.deinitialize(WinFwCleanupPolicy::ResetFirewall);
    }
}

fn settings(permit_lan: bool) -> WinFwSettings {
    WinFwSettings::new(permit_lan)
}

fn no_dhcp_settings(permit_lan: bool) -> WinFwSettings {
    WinFwSettings {
        permit_dhcp: false,
        permit_lan,
    }
}

fn endpoint_v4(ip: &str, port: u16, protocol: WinFwProtocol, clients: &[&str]) -> EndpointArgument {
    EndpointArgument::new(ip, port, protocol, clients)
}

/// A relay over WireGuard's typical UDP port.
const RELAY_V4_UDP: (&str, u16, WinFwProtocol) = ("185.65.134.73", 51820, WinFwProtocol::Udp);
/// A relay reached over TCP.
const RELAY_V4_TCP: (&str, u16, WinFwProtocol) = ("45.83.220.2", 443, WinFwProtocol::Tcp);
/// An IPv6 relay over UDP.
const RELAY_V6_UDP: (&str, u16, WinFwProtocol) = ("2a03:1b20:1:b::4", 51820, WinFwProtocol::Udp);
/// A relay that serves WireGuard over the DNS port, which makes the firewall
/// install its endpoint rule in the DNS sublayer instead of the baseline one.
const RELAY_V4_DNS: (&str, u16, WinFwProtocol) = ("194.242.11.20", 53, WinFwProtocol::Udp);

fn relay((ip, port, protocol): (&str, u16, WinFwProtocol)) -> EndpointArgument {
    endpoint_v4(ip, port, protocol, &[])
}

fn allowed_endpoint(
    ip: &str,
    port: u16,
    protocol: WinFwProtocol,
    clients: &[&str],
) -> EndpointArgument {
    endpoint_v4(ip, port, protocol, clients)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn initialize() {
    let _guard = lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut fw = Implementation::load();
    fw.initialize();

    assert_eq!(fw.active_policy(), WinFwActivePolicy::None);
    assert_snapshot("initialize");

    fw.deinitialize(WinFwCleanupPolicy::ResetFirewall);
    assert_snapshot("initialize-after-deinitialize");
}

// ---------------------------------------------------------------------------
// Blocked policy
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn blocked_policy() {
    let mut fw = Firewall::new();

    for (name, fw_settings, allowed) in [
        ("blocked-no-lan", settings(false), None),
        ("blocked-lan", settings(true), None),
        (
            "blocked-no-lan-allowed-endpoint",
            settings(false),
            Some(allowed_endpoint(
                "193.138.218.74",
                443,
                WinFwProtocol::Tcp,
                &[CLIENT_SVCHOST],
            )),
        ),
        (
            "blocked-lan-allowed-endpoint",
            settings(true),
            Some(allowed_endpoint(
                "193.138.218.74",
                443,
                WinFwProtocol::Tcp,
                &[CLIENT_SVCHOST],
            )),
        ),
        (
            "blocked-allowed-endpoint-no-clients",
            settings(false),
            Some(allowed_endpoint(
                "193.138.218.74",
                443,
                WinFwProtocol::Tcp,
                &[],
            )),
        ),
        (
            "blocked-allowed-endpoint-v6",
            settings(false),
            Some(allowed_endpoint(
                "2a03:1b20:1:d::42",
                443,
                WinFwProtocol::Tcp,
                &[CLIENT_SVCHOST],
            )),
        ),
        (
            "blocked-allowed-endpoint-dns-port",
            settings(false),
            Some(allowed_endpoint(
                "194.242.11.20",
                53,
                WinFwProtocol::Udp,
                &[CLIENT_CMD],
            )),
        ),
        ("blocked-no-dhcp-no-lan", no_dhcp_settings(false), None),
        ("blocked-no-dhcp-lan", no_dhcp_settings(true), None),
    ] {
        let status = fw.apply_policy_blocked(&fw_settings, allowed.as_ref());
        assert_eq!(status, WinFwPolicyStatus::Success, "{name}");
        assert_eq!(fw.active_policy(), WinFwActivePolicy::Blocked, "{name}");
        assert_snapshot(name);
    }

    // Resetting returns to the bare base configuration.
    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_eq!(fw.active_policy(), WinFwActivePolicy::None);
    assert_snapshot("blocked-then-reset");
}

// ---------------------------------------------------------------------------
// Connecting policy
// ---------------------------------------------------------------------------

#[expect(clippy::too_many_arguments)]
fn apply_connecting(
    fw: &mut Firewall,
    name: &str,
    fw_settings: &WinFwSettings,
    relays: &[EndpointArgument],
    exit_endpoint_ip: Option<&str>,
    relay_clients: &[&str],
    tunnel_interface_alias: Option<&str>,
    allowed: &EndpointArgument,
    allowed_tunnel_traffic: AllowedTunnelTraffic<'_>,
) {
    let status = fw.apply_policy_connecting(
        fw_settings,
        relays,
        exit_endpoint_ip,
        relay_clients,
        tunnel_interface_alias,
        allowed,
        allowed_tunnel_traffic,
    );
    assert_eq!(status, WinFwPolicyStatus::Success, "{name}");
    assert_eq!(fw.active_policy(), WinFwActivePolicy::Connecting, "{name}");
    assert_snapshot(name);
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn connecting_policy() {
    let mut fw = Firewall::new();

    let allowed_all = allowed_endpoint("193.138.218.74", 443, WinFwProtocol::Tcp, &[]);

    // The bare minimum: one relay, no restrictions lifted.
    apply_connecting(
        &mut fw,
        "connecting-bare",
        &settings(false),
        &[relay(RELAY_V4_UDP)],
        None,
        &[],
        None,
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::None,
            endpoint1: None,
            endpoint2: None,
        },
    );

    // Private networks reachable.
    apply_connecting(
        &mut fw,
        "connecting-lan",
        &settings(true),
        &[relay(RELAY_V4_UDP)],
        None,
        &[],
        None,
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::None,
            endpoint1: None,
            endpoint2: None,
        },
    );

    let traffic = |traffic_type, endpoint1, endpoint2| AllowedTunnelTraffic {
        traffic_type,
        endpoint1,
        endpoint2,
    };
    use winfw::cpp_ffi::WinFwAllowedTunnelTrafficType as Traffic;

    // The works: multiple relays, per-process relay access, an exit relay for
    // multihop, a tunnel interface with all traffic allowed through it, and an
    // allowed endpoint outside the tunnel.
    let relays = [
        relay(RELAY_V4_UDP),
        relay(RELAY_V4_TCP),
        relay(RELAY_V6_UDP),
    ];
    apply_connecting(
        &mut fw,
        "connecting-multi-relay-clients-tunnel-all",
        &settings(true),
        &relays,
        Some("45.83.220.3"),
        &[CLIENT_SVCHOST, CLIENT_CMD],
        Some(&tunnel_alias()),
        &allowed_all,
        traffic(Traffic::All, None, None),
    );

    // A single tunnel endpoint is allowed to communicate.
    let att_one_endpoint = endpoint_v4("10.64.0.1", 51820, WinFwProtocol::Udp, &[]);
    apply_connecting(
        &mut fw,
        "connecting-tunnel-traffic-one-v6",
        &settings(true),
        &relays,
        Some("45.83.220.3"),
        &[CLIENT_SVCHOST],
        Some(&tunnel_alias()),
        &allowed_all,
        traffic(Traffic::One, Some(&att_one_endpoint), None),
    );

    // Two tunnel endpoints are allowed to communicate.
    let att_two_endpoints = (
        endpoint_v4("10.64.0.1", 51820, WinFwProtocol::Udp, &[]),
        endpoint_v4("10.65.0.1", 51820, WinFwProtocol::Udp, &[]),
    );
    apply_connecting(
        &mut fw,
        "connecting-tunnel-traffic-two",
        &settings(true),
        &relays,
        Some("2a03:1b20:1:b::5"),
        &[CLIENT_SVCHOST],
        Some(&tunnel_alias()),
        &allowed_all,
        traffic(
            Traffic::Two,
            Some(&att_two_endpoints.0),
            Some(&att_two_endpoints.1),
        ),
    );

    // No tunnel interface: the allowed tunnel traffic is ignored entirely.
    apply_connecting(
        &mut fw,
        "connecting-no-tunnel",
        &settings(true),
        &[relay(RELAY_V4_UDP)],
        Some("45.83.220.3"),
        &[CLIENT_SVCHOST],
        None,
        &allowed_all,
        traffic(
            Traffic::Two,
            Some(&att_two_endpoints.0),
            Some(&att_two_endpoints.1),
        ),
    );

    // A relay on the DNS port: its endpoint rule goes in the DNS sublayer.
    apply_connecting(
        &mut fw,
        "connecting-relay-on-dns-port",
        &settings(false),
        &[relay(RELAY_V4_DNS)],
        None,
        &[CLIENT_SVCHOST],
        None,
        &allowed_all,
        traffic(Traffic::None, None, None),
    );

    // An allowed endpoint on the DNS port: its rule goes in the DNS sublayer.
    let allowed_dns_port =
        allowed_endpoint("194.242.11.20", 53, WinFwProtocol::Udp, &[CLIENT_SVCHOST]);
    apply_connecting(
        &mut fw,
        "connecting-allowed-endpoint-on-dns-port",
        &settings(false),
        &[relay(RELAY_V4_UDP)],
        None,
        &[],
        None,
        &allowed_dns_port,
        traffic(Traffic::None, None, None),
    );

    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("connecting-then-reset");
}

// ---------------------------------------------------------------------------
// Connected policy
// ---------------------------------------------------------------------------

#[expect(clippy::too_many_arguments)]
fn apply_connected(
    fw: &mut Firewall,
    name: &str,
    fw_settings: &WinFwSettings,
    relays: &[EndpointArgument],
    exit_endpoint_ip: Option<&str>,
    relay_clients: &[&str],
    tunnel_dns_servers: &[&str],
    non_tunnel_dns_servers: &[&str],
) {
    let status = fw.apply_policy_connected(
        fw_settings,
        relays,
        exit_endpoint_ip,
        relay_clients,
        tunnel_alias().as_str(),
        tunnel_dns_servers,
        non_tunnel_dns_servers,
    );
    assert_eq!(status, WinFwPolicyStatus::Success, "{name}");
    assert_eq!(fw.active_policy(), WinFwActivePolicy::Connected, "{name}");
    assert_snapshot(name);
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn connected_policy() {
    let mut fw = Firewall::new();

    let relays = [relay(RELAY_V4_UDP)];

    // The bare minimum: no DNS servers, no exit relay.
    apply_connected(
        &mut fw,
        "connected-bare",
        &settings(false),
        &relays,
        None,
        &[],
        &[],
        &[],
    );

    // Private networks reachable, relay access restricted to processes.
    apply_connected(
        &mut fw,
        "connected-lan-clients",
        &settings(true),
        &[relay(RELAY_V4_UDP), relay(RELAY_V6_UDP)],
        None,
        &[CLIENT_SVCHOST, CLIENT_CMD],
        &[],
        &[],
    );

    // DNS servers on the tunnel interface only.
    apply_connected(
        &mut fw,
        "connected-tunnel-dns",
        &settings(false),
        &relays,
        None,
        &[],
        &["10.64.0.1"],
        &[],
    );

    // DNS servers outside the tunnel only.
    apply_connected(
        &mut fw,
        "connected-non-tunnel-dns",
        &settings(false),
        &relays,
        None,
        &[],
        &[],
        &["193.138.218.74"],
    );

    // Both, with an IPv6 tunnel resolver, an IPv6 exit relay and an IPv6 relay.
    apply_connected(
        &mut fw,
        "connected-both-dns-v6-exit",
        &settings(true),
        &[relay(RELAY_V6_UDP)],
        Some("2a03:1b20:1:b::5"),
        &[CLIENT_SVCHOST],
        &["fc00::53", "10.64.0.2"],
        &["193.138.218.74", "2a01:3a0:53:53::1"],
    );

    // A relay on the DNS port: its endpoint rule goes in the DNS sublayer.
    apply_connected(
        &mut fw,
        "connected-relay-on-dns-port",
        &settings(false),
        &[relay(RELAY_V4_DNS)],
        None,
        &[CLIENT_CMD],
        &["10.64.0.1"],
        &["193.138.218.74"],
    );

    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("connected-then-reset");
}

// ---------------------------------------------------------------------------
// Reset and deinitialize
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn reset_from_every_policy() {
    let mut fw = Firewall::new();

    let allowed_all = allowed_endpoint("193.138.218.74", 443, WinFwProtocol::Tcp, &[]);
    let relays = [relay(RELAY_V4_UDP)];

    let _ = fw.apply_policy_blocked(&settings(true), Some(&allowed_all));
    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_eq!(fw.active_policy(), WinFwActivePolicy::None);
    assert_snapshot("reset-from-blocked");

    let _ = fw.apply_policy_connecting(
        &settings(true),
        &relays,
        None,
        &[],
        Some(&tunnel_alias()),
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::All,
            endpoint1: None,
            endpoint2: None,
        },
    );
    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("reset-from-connecting");

    let _ = fw.apply_policy_connected(
        &settings(true),
        &relays,
        None,
        &[],
        tunnel_alias().as_str(),
        &["10.64.0.1"],
        &[],
    );
    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("reset-from-connected");
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn deinitialize_from_connected_resets_to_base() {
    let _guard = lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut fw = Implementation::load();
    fw.initialize();

    let relays = [relay(RELAY_V4_UDP)];
    let status = fw.apply_policy_connected(
        &settings(true),
        &relays,
        None,
        &[],
        tunnel_alias().as_str(),
        &["10.64.0.1"],
        &[],
    );
    assert_eq!(status, WinFwPolicyStatus::Success);

    fw.deinitialize(WinFwCleanupPolicy::ResetFirewall);
    // Deinitializing with a reset reverts to the base configuration: no filters,
    // but the provider and sublayers remain until the process resets them.
    assert_snapshot("deinitialize-reset-from-connected");

    // A reset with no initialized module purges everything the firewall ever
    // registered, including the provider and sublayers.
    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("reset-when-uninitialized");
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn initialize_blocked_states() {
    let _guard = lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    for (name, fw_settings, allowed) in [
        ("initialize-blocked", settings(false), None),
        ("initialize-blocked-lan", settings(true), None),
        (
            "initialize-blocked-allowed-endpoint",
            settings(false),
            Some(allowed_endpoint(
                "193.138.218.74",
                443,
                WinFwProtocol::Tcp,
                &[CLIENT_SVCHOST],
            )),
        ),
    ] {
        let mut fw = Implementation::load();
        fw.initialize_blocked(&fw_settings, allowed.as_ref());
        assert_eq!(fw.active_policy(), WinFwActivePolicy::Blocked, "{name}");
        assert_snapshot(name);

        // Leaving blocked filters in place on deinitialize: the state is the
        // same, the objects are simply not removed.
        fw.deinitialize(WinFwCleanupPolicy::BlockingUntilReboot);
        assert_snapshot(&format!("{name}-after-deinitialize-block-until-reboot"));

        // The uninitialized reset purges everything, cleaning up for the next
        // iteration of the loop.
        let status = fw.reset();
        assert_eq!(status, WinFwPolicyStatus::Success);
        assert_snapshot(&format!("{name}-after-reset-when-uninitialized"));
    }
}

// ---------------------------------------------------------------------------
// Transitions and failure behavior
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn policy_transitions() {
    let mut fw = Firewall::new();

    let allowed_all = allowed_endpoint("193.138.218.74", 443, WinFwProtocol::Tcp, &[]);
    let relays = [relay(RELAY_V4_UDP)];

    let _ = fw.apply_policy_blocked(&settings(true), Some(&allowed_all));
    assert_snapshot("transition-blocked");

    let status = fw.apply_policy_connecting(
        &settings(true),
        &relays,
        None,
        &[],
        Some(&tunnel_alias()),
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::All,
            endpoint1: None,
            endpoint2: None,
        },
    );
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("transition-connecting-after-blocked");

    let status = fw.apply_policy_connected(
        &settings(true),
        &relays,
        None,
        &[],
        tunnel_alias().as_str(),
        &["10.64.0.1"],
        &[],
    );
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("transition-connected-after-connecting");

    // Applying the same policy again is idempotent.
    let status = fw.apply_policy_connected(
        &settings(true),
        &relays,
        None,
        &[],
        tunnel_alias().as_str(),
        &["10.64.0.1"],
        &[],
    );
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("transition-connected-again");

    let status = fw.apply_policy_blocked(&settings(true), None);
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("transition-blocked-after-connected");

    let status = fw.apply_policy_connecting(
        &settings(true),
        &relays,
        None,
        &[],
        None,
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::None,
            endpoint1: None,
            endpoint2: None,
        },
    );
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("transition-connecting-after-blocked-no-tunnel");

    let status = fw.reset();
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("transition-reset");
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn failed_apply_leaves_state_untouched() {
    let mut fw = Firewall::new();

    let status = fw.apply_policy_blocked(&settings(true), None);
    assert_eq!(status, WinFwPolicyStatus::Success);
    assert_snapshot("failure-baseline-blocked");

    let allowed_all = allowed_endpoint("193.138.218.74", 443, WinFwProtocol::Tcp, &[]);

    // A relay that equals the exit endpoint is rejected.
    let relays = [relay(RELAY_V4_UDP)];
    let status = fw.apply_policy_connecting(
        &settings(true),
        &relays,
        Some(RELAY_V4_UDP.0),
        &[],
        None,
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::None,
            endpoint1: None,
            endpoint2: None,
        },
    );
    assert_eq!(status, WinFwPolicyStatus::GeneralFailure);

    // A tunnel interface that does not exist is rejected.
    let status = fw.apply_policy_connecting(
        &settings(true),
        &relays,
        None,
        &[],
        Some("Nonexistent-Adapter-42"),
        &allowed_all,
        AllowedTunnelTraffic {
            traffic_type: winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::All,
            endpoint1: None,
            endpoint2: None,
        },
    );
    assert_eq!(status, WinFwPolicyStatus::GeneralFailure);

    assert_eq!(fw.active_policy(), WinFwActivePolicy::Blocked);
    assert_snapshot("failure-state-unchanged");
}

// The harness module must be linked in; it exposes the dumper used above.
#[test]
fn harness_links() {
    assert!(wfp_state::dump().is_ok());
}
