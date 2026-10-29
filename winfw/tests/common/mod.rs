//! Shared harness for the WinFW snapshot tests.
//!
//! The dumper in [`wfp_state`] serializes the complete set of WFP objects the
//! firewall owns into a stable, human-readable textual form. Snapshot tests
//! drive the firewall through states, comparing the dumper output at every
//! step against the snapshots in `tests/snapshots/`.
//!
//! The snapshots were generated from the former C++ implementation before its
//! removal, and now guard the Rust implementation against behavior drift:
//! every state must install exactly the WFP objects recorded in them.
//!
//! Run with `cargo test` as administrator (graphical session not required).
//! Regenerate with `INSTA_UPDATE=always cargo test -p winfw`, or use
//! `cargo insta test` and review the pending snapshots with
//! `cargo insta review`, only for deliberate, reviewed changes to the
//! firewall rules.

pub mod wfp_state;

use std::{
    net::IpAddr,
    sync::{Mutex, OnceLock},
    time::Duration,
};

/// The firewall implementation under test: the Rust implementation.
pub enum Implementation {
    Rust(Option<winfw::Manager>),
}

impl Implementation {
    /// Load every implementation selected by cargo features.
    pub fn load_all() -> Vec<Self> {
        vec![Self::Rust(None)]
    }

    /// `initialize`.
    pub fn initialize(&mut self) {
        match self {
            Self::Rust(manager) => {
                *manager = Some(
                    winfw::Manager::new(Duration::from_secs(u64::from(super::TIMEOUT_SECONDS)))
                        .expect("Failed to initialize the firewall"),
                );
            }
        }
    }

    /// `initialize_blocked`.
    pub fn initialize_blocked(
        &mut self,
        settings: &Settings,
        allowed_endpoint: Option<&EndpointArgument>,
    ) {
        match self {
            Self::Rust(manager) => {
                *manager = Some(
                    winfw::Manager::new_blocked(
                        Duration::from_secs(u64::from(super::TIMEOUT_SECONDS)),
                        &to_settings(settings),
                        allowed_endpoint.map(to_allowed_endpoint).as_ref(),
                    )
                    .expect("Failed to initialize the firewall to a blocking state"),
                );
            }
        }
    }

    /// `deinitialize`.
    pub fn deinitialize(&mut self, cleanup: CleanupPolicy) {
        match self {
            Self::Rust(manager) => {
                if let Some(mut manager) = manager.take() {
                    manager
                        .deinitialize(to_cleanup_policy(cleanup))
                        .expect("Failed to deinitialize the firewall");
                }
            }
        }
    }

    /// `active_policy`.
    pub fn active_policy(&self) -> ActivePolicy {
        match self {
            Self::Rust(manager) => match manager.as_ref().map(winfw::Manager::active_policy) {
                Some(winfw::ActivePolicy::None) | None => ActivePolicy::None,
                Some(winfw::ActivePolicy::Connecting) => ActivePolicy::Connecting,
                Some(winfw::ActivePolicy::Connected) => ActivePolicy::Connected,
                Some(winfw::ActivePolicy::Blocked) => ActivePolicy::Blocked,
            },
        }
    }

    /// `apply_policy_connecting`.
    #[expect(clippy::too_many_arguments)]
    pub fn apply_policy_connecting(
        &mut self,
        settings: &Settings,
        relays: &[EndpointArgument],
        exit_endpoint_ip: Option<&str>,
        relay_clients: &[&str],
        tunnel_interface_alias: Option<&str>,
        allowed_endpoint: &EndpointArgument,
        allowed_tunnel_traffic: AllowedTunnelTraffic<'_>,
    ) -> PolicyStatus {
        match self {
            Self::Rust(manager) => {
                let Some(manager) = manager.as_mut() else {
                    return PolicyStatus::GeneralFailure;
                };
                let tunnel = winfw::TunnelParameters {
                    alias: tunnel_interface_alias.map(|alias| alias.to_owned()),
                    exit_ip: exit_endpoint_ip.map(|ip| ip.parse().expect("valid IP")),
                    clients: relay_clients
                        .iter()
                        .map(|client| client.to_string())
                        .collect(),
                    allowed_tunnel_traffic: match allowed_tunnel_traffic.traffic_type {
                        TrafficType::None => winfw::AllowedTunnelTraffic::None,
                        TrafficType::All => winfw::AllowedTunnelTraffic::All,
                        TrafficType::One => winfw::AllowedTunnelTraffic::One(to_relay_endpoint(
                            allowed_tunnel_traffic.endpoint1.expect("endpoint one"),
                        )),
                        TrafficType::Two => winfw::AllowedTunnelTraffic::Two(
                            to_relay_endpoint(
                                allowed_tunnel_traffic.endpoint1.expect("endpoint one"),
                            ),
                            to_relay_endpoint(
                                allowed_tunnel_traffic.endpoint2.expect("endpoint two"),
                            ),
                        ),
                    },
                };
                let status = manager.apply_policy_connecting(
                    &to_settings(settings),
                    &to_relay_endpoints(relays),
                    &tunnel,
                    &to_allowed_endpoint(allowed_endpoint),
                );
                if let Err(error) = &status {
                    eprintln!("[rust-impl] connecting policy failed: {error:?}");
                }
                to_status(status)
            }
        }
    }

    /// `apply_policy_connected`.
    #[expect(clippy::too_many_arguments)]
    pub fn apply_policy_connected(
        &mut self,
        settings: &Settings,
        relays: &[EndpointArgument],
        exit_endpoint_ip: Option<&str>,
        relay_clients: &[&str],
        tunnel_interface_alias: &str,
        tunnel_dns_servers: &[&str],
        non_tunnel_dns_servers: &[&str],
    ) -> PolicyStatus {
        match self {
            Self::Rust(manager) => {
                let Some(manager) = manager.as_mut() else {
                    return PolicyStatus::GeneralFailure;
                };
                let tunnel = winfw::TunnelParameters {
                    alias: Some(tunnel_interface_alias.to_owned()),
                    exit_ip: exit_endpoint_ip.map(|ip| ip.parse().expect("valid IP")),
                    clients: relay_clients
                        .iter()
                        .map(|client| client.to_string())
                        .collect(),
                    allowed_tunnel_traffic: winfw::AllowedTunnelTraffic::All,
                };
                let status = manager.apply_policy_connected(
                    &to_settings(settings),
                    &to_relay_endpoints(relays),
                    &tunnel,
                    &tunnel_dns_servers
                        .iter()
                        .map(|ip| ip.parse::<IpAddr>().expect("valid IP"))
                        .collect::<Vec<_>>(),
                    &non_tunnel_dns_servers
                        .iter()
                        .map(|ip| ip.parse::<IpAddr>().expect("valid IP"))
                        .collect::<Vec<_>>(),
                );
                if let Err(error) = &status {
                    eprintln!("[rust-impl] connected policy failed: {error:?}");
                }
                to_status(status)
            }
        }
    }

    /// `apply_policy_blocked`.
    pub fn apply_policy_blocked(
        &mut self,
        settings: &Settings,
        allowed_endpoint: Option<&EndpointArgument>,
    ) -> PolicyStatus {
        match self {
            Self::Rust(manager) => {
                let Some(manager) = manager.as_mut() else {
                    return PolicyStatus::GeneralFailure;
                };
                let status = manager.apply_policy_blocked(
                    &to_settings(settings),
                    allowed_endpoint.map(to_allowed_endpoint).as_ref(),
                );
                if let Err(error) = &status {
                    eprintln!("[rust-impl] blocked policy failed: {error:?}");
                }
                to_status(status)
            }
        }
    }

    /// `reset`.
    pub fn reset(&mut self) -> PolicyStatus {
        match self {
            Self::Rust(manager) => match manager.as_mut() {
                Some(manager) => {
                    let result = manager.reset();
                    if let Err(error) = &result {
                        eprintln!("[rust-impl] reset failed: {error:?}");
                    }
                    to_status(result)
                }
                // Reset with no initialized module purges everything.
                None => match winfw::purge_all() {
                    Ok(()) => PolicyStatus::Success,
                    Err(_) => PolicyStatus::GeneralFailure,
                },
            },
        }
    }
}

/// An endpoint argument: an IP, port and protocol, with the allowed client
/// processes.
pub struct EndpointArgument {
    ip_str: String,
    clients_str: Vec<String>,
    port: u16,
    protocol: Protocol,
}

impl EndpointArgument {
    pub fn new(ip: &str, port: u16, protocol: Protocol, clients: &[&str]) -> Self {
        Self {
            ip_str: ip.to_owned(),
            clients_str: clients.iter().map(|client| client.to_string()).collect(),
            port,
            protocol,
        }
    }

    /// The IP address, as passed in.
    pub fn ip(&self) -> &str {
        &self.ip_str
    }

    /// The port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The transport protocol.
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// The allowed client processes, as passed in.
    pub fn clients(&self) -> &[String] {
        &self.clients_str
    }
}

/// Allowed tunnel traffic argument.
pub struct AllowedTunnelTraffic<'a> {
    pub traffic_type: TrafficType,
    pub endpoint1: Option<&'a EndpointArgument>,
    pub endpoint2: Option<&'a EndpointArgument>,
}

/// The traffic allowed in the tunnel while connecting.
pub enum TrafficType {
    None,
    All,
    One,
    Two,
}

/// Whether DHCP requests are permitted, and private networks are reachable.
pub struct Settings {
    pub permit_dhcp: bool,
    pub permit_lan: bool,
}

impl Settings {
    pub fn new(permit_lan: bool) -> Self {
        Self {
            permit_dhcp: true,
            permit_lan,
        }
    }
}

/// The policy currently in effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivePolicy {
    None,
    Connecting,
    Connected,
    Blocked,
}

/// How to clean up when deinitializing.
pub enum CleanupPolicy {
    ResetFirewall,
    BlockingUntilReboot,
}

/// Whether a policy application succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyStatus {
    Success,
    GeneralFailure,
    LockTimeout,
}

/// The transport protocol of an endpoint argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// The alias of an existing interface to use for the tunnel rules. The firewall
/// only resolves the alias to a LUID, so any present interface works; the
/// snapshots render it as a placeholder and stay portable. Override with
/// `TUNNEL_ALIAS`.
pub fn tunnel_alias() -> String {
    static ALIAS: OnceLock<String> = OnceLock::new();
    ALIAS.get_or_init(find_tunnel_alias).clone()
}

fn find_tunnel_alias() -> String {
    if let Ok(alias) = std::env::var("TUNNEL_ALIAS") {
        return alias;
    }

    use windows_sys::Win32::NetworkManagement::IpHelper::GetAdaptersAddresses;
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211, IP_ADAPTER_ADDRESSES_LH,
    };
    use windows_sys::Win32::Networking::WinSock::AF_UNSPEC as AF_UNSPEC_WSK;

    const GAA_FLAG_SKIP_UNICAST: u32 = 0x0001;
    const GAA_FLAG_SKIP_ANYCAST: u32 = 0x0002;
    const GAA_FLAG_SKIP_MULTICAST: u32 = 0x0010;
    const GAA_FLAG_SKIP_DNS_SERVER: u32 = 0x0008;
    const FLAGS: u32 = GAA_FLAG_SKIP_UNICAST
        | GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST
        | GAA_FLAG_SKIP_DNS_SERVER;

    let mut buffer = vec![0u8; 16 * 1024];
    let mut needed = 0u32;
    // SAFETY: `buffer` is at least `needed` bytes on every iteration, `needed` is
    // a valid out-param, and the loop retries with the size the API asks for.
    let mut status = unsafe {
        GetAdaptersAddresses(
            AF_UNSPEC_WSK as u32,
            FLAGS,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &raw mut needed,
        )
    };
    const ERROR_BUFFER_OVERFLOW: u32 = 111;
    if status == ERROR_BUFFER_OVERFLOW {
        buffer = vec![0u8; needed as usize];
        // SAFETY: see above.
        status = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC_WSK as u32,
                FLAGS,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &raw mut needed,
            )
        };
    }
    if status != 0 {
        panic!("GetAdaptersAddresses failed with error {status}");
    }

    // SAFETY: the buffer now holds an `IP_ADAPTER_ADDRESSES_LH` linked list.
    let mut current = buffer.as_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
    while !current.is_null() {
        // SAFETY: the list node lives in `buffer` for the duration of this loop.
        let adapter = unsafe { &*current };
        if adapter.IfType == IF_TYPE_ETHERNET_CSMACD || adapter.IfType == IF_TYPE_IEEE80211 {
            // SAFETY: `FriendlyName` is a null-terminated wide string.
            let alias = unsafe {
                let mut len = 0usize;
                while *adapter.FriendlyName.add(len) != 0 {
                    len += 1;
                }
                String::from_utf16_lossy(std::slice::from_raw_parts(adapter.FriendlyName, len))
            };
            if !alias.is_empty() {
                return alias;
            }
        }
        current = adapter.Next;
    }

    panic!("No Ethernet or Wi-Fi interface found for the tunnel alias");
}

/// Compare the current WFP state against the snapshot `name`.
///
/// The snapshots live in `tests/snapshots/{name}.snap` in insta's file
/// format, with the module prefix disabled in the bound settings so the names
/// stay exactly the state names. Update them with
/// `INSTA_UPDATE=always cargo test -p winfw`, or review pending ones with
/// `cargo insta review`.
pub fn assert_snapshot(name: &str) {
    let actual = wfp_state::dump().expect("Failed to dump WFP state");

    let mut settings = insta::Settings::clone_current();
    // Without this, insta would prefix the snapshot file names with the module
    // path of this wrapper (`common__`), not with the test asking for it.
    settings.set_prepend_module_to_snapshot(false);
    // And it would also store them next to this file, like it does for
    // assertions made directly in a test. Point it at the crate's snapshot
    // directory explicitly.
    settings.set_snapshot_path(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots"));
    settings.bind(|| {
        insta::assert_snapshot!(name, actual);
    });
}
fn to_settings(settings: &Settings) -> winfw::Settings {
    winfw::Settings {
        permit_dhcp: settings.permit_dhcp,
        permit_lan: settings.permit_lan,
    }
}

fn to_protocol(protocol: Protocol) -> winfw::Protocol {
    match protocol {
        Protocol::Tcp => winfw::Protocol::Tcp,
        Protocol::Udp => winfw::Protocol::Udp,
    }
}

fn to_relay_endpoint(endpoint: &EndpointArgument) -> winfw::RelayEndpoint {
    winfw::RelayEndpoint {
        ip: endpoint.ip().parse().expect("valid IP"),
        port: endpoint.port(),
        protocol: to_protocol(endpoint.protocol()),
    }
}

fn to_relay_endpoints(relays: &[EndpointArgument]) -> Vec<winfw::RelayEndpoint> {
    relays.iter().map(to_relay_endpoint).collect()
}

fn to_allowed_endpoint(endpoint: &EndpointArgument) -> winfw::AllowedEndpoint {
    winfw::AllowedEndpoint {
        clients: endpoint.clients().to_vec(),
        endpoint: to_relay_endpoint(endpoint),
    }
}

fn to_cleanup_policy(cleanup: CleanupPolicy) -> winfw::CleanupPolicy {
    match cleanup {
        CleanupPolicy::ResetFirewall => winfw::CleanupPolicy::ResetFirewall,
        CleanupPolicy::BlockingUntilReboot => winfw::CleanupPolicy::BlockingUntilReboot,
    }
}

fn to_status(result: Result<(), winfw::ApplyError>) -> PolicyStatus {
    match result {
        Ok(()) => PolicyStatus::Success,
        Err(winfw::ApplyError::LockTimeout(_)) => PolicyStatus::LockTimeout,
        Err(winfw::ApplyError::Generic(_)) => PolicyStatus::GeneralFailure,
    }
}

/// Serialize all tests: the firewall library is a per-process singleton and
/// the WFP state it holds is global.
fn lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// A driver that owns an implementation: initializes it on creation and
/// deinitializes it (with a full reset) when dropped, even if a test panics.
pub struct Firewall {
    implementation: Implementation,
}

impl Firewall {
    fn new(implementation: Implementation, initialize: bool) -> Self {
        let mut driver = Self { implementation };
        if initialize {
            driver.initialize();
        }
        driver
    }
}

impl std::ops::Deref for Firewall {
    type Target = Implementation;

    fn deref(&self) -> &Self::Target {
        &self.implementation
    }
}

impl std::ops::DerefMut for Firewall {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.implementation
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        self.implementation
            .deinitialize(CleanupPolicy::ResetFirewall);
    }
}

/// Run a test body once per implementation, each initialized and torn down as
/// its own firewall session, asserting the same snapshots from each.
pub fn run_test(body: impl Fn(&mut Firewall)) {
    let _guard = lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for implementation in Implementation::load_all() {
        let mut driver = Firewall::new(implementation, true);
        body(&mut driver);
    }
}

/// Run a test body once per implementation, without initializing it first:
/// for tests that drive the blocked initialization and the uninitialized
/// reset themselves.
pub fn run_test_uninitialized(body: impl Fn(&mut Firewall)) {
    let _guard = lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for implementation in Implementation::load_all() {
        let mut driver = Firewall::new(implementation, false);
        body(&mut driver);
    }
}
