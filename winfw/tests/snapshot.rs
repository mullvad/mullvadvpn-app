#![cfg(target_os = "windows")]

//! Snapshot tests for every WinFW firewall state.
//!
//! Each test drives the firewall through one or more states, dumping the
//! complete WFP object state after each step and comparing it with the
//! snapshots in `tests/snapshots/`, guarding the firewall against behavior
//! drift.
//!
//! The tests are ignored by default: they replace the WFP objects of any
//! running Mullvad daemon, and must run as administrator. See the README for
//! how to run them, and how to update the snapshots.

mod common;

use std::net::IpAddr;

use common::{
    Firewall, allowed_endpoint, assert_snapshot, endpoint, run_test, run_test_uninitialized,
};
use winfw::{
    ActivePolicy, AllowedTunnelTraffic, ApplyError, CleanupPolicy, Protocol, RelayEndpoint,
    Settings, TunnelParameters,
};

/// Client processes that exist on every Windows installation, used to exercise
/// the per-application firewall rules. The full paths are passed to the firewall,
/// which resolves them to WFP app identifiers.
const CLIENT_SVCHOST: &str = r"C:\Windows\System32\svchost.exe";
const CLIENT_CMD: &str = r"C:\Windows\System32\cmd.exe";

/// A relay over WireGuard's typical UDP port.
const RELAY_V4_UDP: (&str, u16, Protocol) = ("185.65.134.73", 51820, Protocol::Udp);
/// A relay reached over TCP.
const RELAY_V4_TCP: (&str, u16, Protocol) = ("45.83.220.2", 443, Protocol::Tcp);
/// An IPv6 relay over UDP.
const RELAY_V6_UDP: (&str, u16, Protocol) = ("2a03:1b20:1:b::4", 51820, Protocol::Udp);
/// A relay that serves WireGuard over the DNS port, which makes the firewall
/// install its endpoint rule in the DNS sublayer instead of the baseline one.
const RELAY_V4_DNS: (&str, u16, Protocol) = ("194.242.11.20", 53, Protocol::Udp);

fn relay((ip, port, protocol): (&str, u16, Protocol)) -> RelayEndpoint {
    endpoint(ip, port, protocol)
}

fn no_dhcp_settings(permit_lan: bool) -> Settings {
    Settings {
        permit_dhcp: false,
        permit_lan,
    }
}

fn tunnel(
    alias: Option<&str>,
    exit_ip: Option<&str>,
    clients: &[&str],
    allowed_tunnel_traffic: AllowedTunnelTraffic,
) -> TunnelParameters {
    TunnelParameters {
        alias: alias.map(str::to_owned),
        exit_ip: exit_ip.map(|ip| ip.parse().expect("valid IP")),
        clients: clients.iter().map(Into::into).collect(),
        allowed_tunnel_traffic,
    }
}

fn ips(ips: &[&str]) -> Vec<IpAddr> {
    ips.iter().map(|ip| ip.parse().expect("valid IP")).collect()
}

/// Apply the connected policy, with the tunnel on the test's tunnel interface.
fn apply_connected(
    fw: &mut Firewall,
    settings: &Settings,
    relays: &[RelayEndpoint],
    exit_ip: Option<&str>,
    clients: &[&str],
    tunnel_dns_servers: &[&str],
    non_tunnel_dns_servers: &[&str],
) {
    let alias = common::tunnel_alias();
    fw.manager()
        .apply_policy_connected(
            settings,
            relays,
            &tunnel(Some(&alias), exit_ip, clients, AllowedTunnelTraffic::All),
            &ips(tunnel_dns_servers),
            &ips(non_tunnel_dns_servers),
        )
        .expect("Failed to apply the connected policy");
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn initialize() {
    run_test(|fw| {
        assert_eq!(fw.active_policy(), ActivePolicy::None);
        assert_snapshot("initialize");

        fw.deinitialize(CleanupPolicy::ResetFirewall);
        assert_snapshot("initialize-after-deinitialize");
    });
}

// ---------------------------------------------------------------------------
// Blocked policy
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn blocked_policy() {
    run_test(|fw| {
        fw.manager()
            .apply_policy_blocked(&Settings::new(false), None)
            .expect("Failed to apply the blocked policy");
        assert_eq!(fw.active_policy(), ActivePolicy::Blocked);
        assert_snapshot("blocked-no-lan");

        fw.manager()
            .apply_policy_blocked(&Settings::new(true), None)
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-lan");

        fw.manager()
            .apply_policy_blocked(
                &Settings::new(false),
                Some(&allowed_endpoint(
                    "193.138.218.74",
                    443,
                    Protocol::Tcp,
                    &[CLIENT_SVCHOST],
                )),
            )
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-no-lan-allowed-endpoint");

        fw.manager()
            .apply_policy_blocked(
                &Settings::new(true),
                Some(&allowed_endpoint(
                    "193.138.218.74",
                    443,
                    Protocol::Tcp,
                    &[CLIENT_SVCHOST],
                )),
            )
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-lan-allowed-endpoint");

        fw.manager()
            .apply_policy_blocked(
                &Settings::new(false),
                Some(&allowed_endpoint("193.138.218.74", 443, Protocol::Tcp, &[])),
            )
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-allowed-endpoint-no-clients");

        fw.manager()
            .apply_policy_blocked(
                &Settings::new(false),
                Some(&allowed_endpoint(
                    "2a03:1b20:1:d::42",
                    443,
                    Protocol::Tcp,
                    &[CLIENT_SVCHOST],
                )),
            )
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-allowed-endpoint-v6");

        fw.manager()
            .apply_policy_blocked(
                &Settings::new(false),
                Some(&allowed_endpoint(
                    "194.242.11.20",
                    53,
                    Protocol::Udp,
                    &[CLIENT_CMD],
                )),
            )
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-allowed-endpoint-dns-port");

        fw.manager()
            .apply_policy_blocked(&no_dhcp_settings(false), None)
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-no-dhcp-no-lan");

        fw.manager()
            .apply_policy_blocked(&no_dhcp_settings(true), None)
            .expect("Failed to apply the blocked policy");
        assert_snapshot("blocked-no-dhcp-lan");

        // Resetting returns to the bare base configuration.
        fw.reset().expect("Failed to reset");
        assert_eq!(fw.active_policy(), ActivePolicy::None);
        assert_snapshot("blocked-then-reset");
    });
}

// ---------------------------------------------------------------------------
// Connecting policy
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn connecting_policy() {
    run_test(|fw| {
        let alias = common::tunnel_alias();
        let allowed_all = allowed_endpoint("193.138.218.74", 443, Protocol::Tcp, &[]);
        let allowed_dns_port =
            allowed_endpoint("194.242.11.20", 53, Protocol::Udp, &[CLIENT_SVCHOST]);
        let relays = [
            relay(RELAY_V4_UDP),
            relay(RELAY_V4_TCP),
            relay(RELAY_V6_UDP),
        ];
        let tunnel_endpoints = (
            endpoint("10.64.0.1", 51820, Protocol::Udp),
            endpoint("10.65.0.1", 51820, Protocol::Udp),
        );

        // The bare minimum: one relay, no restrictions lifted.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(false),
                &[relay(RELAY_V4_UDP)],
                &tunnel(None, None, &[], AllowedTunnelTraffic::None),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_eq!(fw.active_policy(), ActivePolicy::Connecting);
        assert_snapshot("connecting-bare");

        // Private networks reachable.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &[relay(RELAY_V4_UDP)],
                &tunnel(None, None, &[], AllowedTunnelTraffic::None),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-lan");

        // The works: multiple relays, per-process relay access, an exit relay
        // for multihop, a tunnel interface with all traffic allowed through it,
        // and an allowed endpoint outside the tunnel.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &relays,
                &tunnel(
                    Some(&alias),
                    Some("45.83.220.3"),
                    &[CLIENT_SVCHOST, CLIENT_CMD],
                    AllowedTunnelTraffic::All,
                ),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-multi-relay-clients-tunnel-all");

        // A single tunnel endpoint is allowed to communicate.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &relays,
                &tunnel(
                    Some(&alias),
                    Some("45.83.220.3"),
                    &[CLIENT_SVCHOST],
                    AllowedTunnelTraffic::One(tunnel_endpoints.0.clone()),
                ),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-tunnel-traffic-one-v6");

        // Two tunnel endpoints are allowed to communicate.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &relays,
                &tunnel(
                    Some(&alias),
                    Some("2a03:1b20:1:b::5"),
                    &[CLIENT_SVCHOST],
                    AllowedTunnelTraffic::Two(
                        tunnel_endpoints.0.clone(),
                        tunnel_endpoints.1.clone(),
                    ),
                ),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-tunnel-traffic-two");

        // No tunnel interface: the allowed tunnel traffic is ignored entirely.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &[relay(RELAY_V4_UDP)],
                &tunnel(
                    None,
                    Some("45.83.220.3"),
                    &[CLIENT_SVCHOST],
                    AllowedTunnelTraffic::Two(
                        tunnel_endpoints.0.clone(),
                        tunnel_endpoints.1.clone(),
                    ),
                ),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-no-tunnel");

        // A relay on the DNS port: its endpoint rule goes in the DNS sublayer.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(false),
                &[relay(RELAY_V4_DNS)],
                &tunnel(None, None, &[CLIENT_SVCHOST], AllowedTunnelTraffic::None),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-relay-on-dns-port");

        // An allowed endpoint on the DNS port: its rule goes in the DNS sublayer.
        fw.manager()
            .apply_policy_connecting(
                &Settings::new(false),
                &[relay(RELAY_V4_UDP)],
                &tunnel(None, None, &[], AllowedTunnelTraffic::None),
                &allowed_dns_port,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("connecting-allowed-endpoint-on-dns-port");

        fw.reset().expect("Failed to reset");
        assert_snapshot("connecting-then-reset");
    });
}

// ---------------------------------------------------------------------------
// Connected policy
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn connected_policy() {
    run_test(|fw| {
        let relays = [relay(RELAY_V4_UDP)];

        // The bare minimum: no DNS servers, no exit relay.
        apply_connected(fw, &Settings::new(false), &relays, None, &[], &[], &[]);
        assert_eq!(fw.active_policy(), ActivePolicy::Connected);
        assert_snapshot("connected-bare");

        // Private networks reachable, relay access restricted to processes.
        apply_connected(
            fw,
            &Settings::new(true),
            &[relay(RELAY_V4_UDP), relay(RELAY_V6_UDP)],
            None,
            &[CLIENT_SVCHOST, CLIENT_CMD],
            &[],
            &[],
        );
        assert_snapshot("connected-lan-clients");

        // DNS servers on the tunnel interface only.
        apply_connected(
            fw,
            &Settings::new(false),
            &relays,
            None,
            &[],
            &["10.64.0.1"],
            &[],
        );
        assert_snapshot("connected-tunnel-dns");

        // DNS servers outside the tunnel only.
        apply_connected(
            fw,
            &Settings::new(false),
            &relays,
            None,
            &[],
            &[],
            &["193.138.218.74"],
        );
        assert_snapshot("connected-non-tunnel-dns");

        // Both, with an IPv6 tunnel resolver, an IPv6 exit relay and an IPv6 relay.
        apply_connected(
            fw,
            &Settings::new(true),
            &[relay(RELAY_V6_UDP)],
            Some("2a03:1b20:1:b::5"),
            &[CLIENT_SVCHOST],
            &["fc00::53", "10.64.0.2"],
            &["193.138.218.74", "2a01:3a0:53:53::1"],
        );
        assert_snapshot("connected-both-dns-v6-exit");

        // A relay on the DNS port: its endpoint rule goes in the DNS sublayer.
        apply_connected(
            fw,
            &Settings::new(false),
            &[relay(RELAY_V4_DNS)],
            None,
            &[CLIENT_CMD],
            &["10.64.0.1"],
            &["193.138.218.74"],
        );
        assert_snapshot("connected-relay-on-dns-port");

        fw.reset().expect("Failed to reset");
        assert_snapshot("connected-then-reset");
    });
}

// ---------------------------------------------------------------------------
// Reset and deinitialize
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn reset_from_every_policy() {
    run_test(|fw| {
        let alias = common::tunnel_alias();
        let allowed_all = allowed_endpoint("193.138.218.74", 443, Protocol::Tcp, &[]);
        let relays = [relay(RELAY_V4_UDP)];

        fw.manager()
            .apply_policy_blocked(&Settings::new(true), Some(&allowed_all))
            .expect("Failed to apply the blocked policy");
        fw.reset().expect("Failed to reset");
        assert_eq!(fw.active_policy(), ActivePolicy::None);
        assert_snapshot("reset-from-blocked");

        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &relays,
                &tunnel(Some(&alias), None, &[], AllowedTunnelTraffic::All),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        fw.reset().expect("Failed to reset");
        assert_snapshot("reset-from-connecting");

        apply_connected(
            fw,
            &Settings::new(true),
            &relays,
            None,
            &[],
            &["10.64.0.1"],
            &[],
        );
        fw.reset().expect("Failed to reset");
        assert_snapshot("reset-from-connected");
    });
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn deinitialize_from_connected_resets_to_base() {
    run_test(|fw| {
        let relays = [relay(RELAY_V4_UDP)];
        apply_connected(
            fw,
            &Settings::new(true),
            &relays,
            None,
            &[],
            &["10.64.0.1"],
            &[],
        );

        // Deinitializing with a reset removes everything the firewall
        // registered, including the provider and sublayers.
        fw.deinitialize(CleanupPolicy::ResetFirewall);
        assert_snapshot("deinitialize-reset-from-connected");

        // A reset with no initialized firewall purges everything it ever
        // registered, which by now is nothing.
        fw.reset().expect("Failed to reset");
        assert_snapshot("reset-when-uninitialized");
    });
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn initialize_blocked_states() {
    run_test_uninitialized(|fw| {
        for (name, settings, allowed) in [
            ("initialize-blocked", Settings::new(false), None),
            ("initialize-blocked-lan", Settings::new(true), None),
            (
                "initialize-blocked-allowed-endpoint",
                Settings::new(false),
                Some(allowed_endpoint(
                    "193.138.218.74",
                    443,
                    Protocol::Tcp,
                    &[CLIENT_SVCHOST],
                )),
            ),
        ] {
            fw.initialize_blocked(&settings, allowed.as_ref());
            assert_eq!(fw.active_policy(), ActivePolicy::Blocked, "{name}");
            assert_snapshot(name);

            // Leaving blocked filters in place on deinitialize: the state is
            // the same, the objects are simply not removed.
            fw.deinitialize(CleanupPolicy::BlockingUntilReboot);
            assert_snapshot(&format!("{name}-after-deinitialize-block-until-reboot"));

            // The uninitialized reset purges everything, cleaning up for the
            // next iteration of the loop.
            fw.reset().expect("Failed to reset");
            assert_snapshot(&format!("{name}-after-reset-when-uninitialized"));
        }
    });
}

// ---------------------------------------------------------------------------
// Transitions and failure behavior
// ---------------------------------------------------------------------------

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn policy_transitions() {
    run_test(|fw| {
        let alias = common::tunnel_alias();
        let allowed_all = allowed_endpoint("193.138.218.74", 443, Protocol::Tcp, &[]);
        let relays = [relay(RELAY_V4_UDP)];

        fw.manager()
            .apply_policy_blocked(&Settings::new(true), Some(&allowed_all))
            .expect("Failed to apply the blocked policy");
        assert_snapshot("transition-blocked");

        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &relays,
                &tunnel(Some(&alias), None, &[], AllowedTunnelTraffic::All),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("transition-connecting-after-blocked");

        apply_connected(
            fw,
            &Settings::new(true),
            &relays,
            None,
            &[],
            &["10.64.0.1"],
            &[],
        );
        assert_snapshot("transition-connected-after-connecting");

        // Applying the same policy again is idempotent.
        apply_connected(
            fw,
            &Settings::new(true),
            &relays,
            None,
            &[],
            &["10.64.0.1"],
            &[],
        );
        assert_snapshot("transition-connected-again");

        fw.manager()
            .apply_policy_blocked(&Settings::new(true), None)
            .expect("Failed to apply the blocked policy");
        assert_snapshot("transition-blocked-after-connected");

        fw.manager()
            .apply_policy_connecting(
                &Settings::new(true),
                &relays,
                &tunnel(None, None, &[], AllowedTunnelTraffic::None),
                &allowed_all,
            )
            .expect("Failed to apply the connecting policy");
        assert_snapshot("transition-connecting-after-blocked-no-tunnel");

        fw.reset().expect("Failed to reset");
        assert_snapshot("transition-reset");
    });
}

#[test]
#[ignore = "replaces the system's Mullvad WFP objects; run as administrator with --ignored"]
fn failed_apply_leaves_state_untouched() {
    run_test(|fw| {
        let allowed_all = allowed_endpoint("193.138.218.74", 443, Protocol::Tcp, &[]);
        let relays = [relay(RELAY_V4_UDP)];

        fw.manager()
            .apply_policy_blocked(&Settings::new(true), None)
            .expect("Failed to apply the blocked policy");
        assert_snapshot("failure-baseline-blocked");

        // A relay that equals the exit endpoint is rejected.
        let result = fw.manager().apply_policy_connecting(
            &Settings::new(true),
            &relays,
            &tunnel(None, Some(RELAY_V4_UDP.0), &[], AllowedTunnelTraffic::None),
            &allowed_all,
        );
        assert!(matches!(result, Err(ApplyError::Generic(_))));

        // A tunnel interface that does not exist is rejected.
        let result = fw.manager().apply_policy_connecting(
            &Settings::new(true),
            &relays,
            &tunnel(
                Some("Nonexistent-Adapter-42"),
                None,
                &[],
                AllowedTunnelTraffic::All,
            ),
            &allowed_all,
        );
        assert!(matches!(result, Err(ApplyError::Generic(_))));

        assert_eq!(fw.active_policy(), ActivePolicy::Blocked);
        assert_snapshot("failure-state-unchanged");
    });
}
