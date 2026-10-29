//! Shared harness for the WinFW snapshot tests.
//!
//! The dumper in [`wfp_state`] serializes the complete set of WFP objects the
//! firewall owns into a stable, human-readable textual form. Snapshot tests
//! drive the firewall implementation through states, comparing the dumper
//! output at every step against the snapshots in `tests/snapshots/`.
//!
//! With both the `cpp-ffi` and `rust` features enabled, every test runs once
//! per implementation, each asserting it installs the same WFP objects as the
//! snapshots say it should.
//!
//! Run with `cargo test` as administrator (graphical session not required).
//! Regenerate with `INSTA_UPDATE=always cargo test -p winfw`, or use
//! `cargo insta test` and review the pending snapshots with
//! `cargo insta review`. Set `WINFW_DLL_PATH` to the C++ `winfw.dll` to use
//! the C++ reference implementation.

pub mod wfp_state;

use std::{
    net::IpAddr,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use winfw::cpp_ffi::{
    AllowedTunnelTraffic, CppWinFw, EndpointArgument, WinFwActivePolicy, WinFwCleanupPolicy,
    WinFwPolicyStatus, WinFwProtocol, WinFwSettings,
};

/// The firewall implementation under test.
pub enum Implementation {
    /// The C++ WinFW library, loaded from `winfw.dll`.
    Cpp(CppWinFw),
    /// The Rust implementation, `winfw::Manager`, kept while initialized.
    #[cfg(feature = "rust")]
    Rust(Option<winfw::Manager>),
}

impl Implementation {
    /// Load every implementation selected by cargo features.
    pub fn load_all() -> Vec<Self> {
        let mut implementations = Vec::new();
        if cfg!(feature = "cpp-ffi") {
            implementations.push(Self::Cpp(
                CppWinFw::load_default().expect("Failed to load winfw.dll"),
            ));
        }
        #[cfg(feature = "rust")]
        {
            implementations.push(Self::Rust(None));
        }
        implementations
    }
}

impl Implementation {
    /// `WinFw_Initialize`.
    pub fn initialize(&mut self) {
        match self {
            Self::Cpp(fw) => fw.initialize(super::TIMEOUT_SECONDS).unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                *manager = Some(
                    winfw::Manager::new(Duration::from_secs(u64::from(super::TIMEOUT_SECONDS)))
                        .expect("Failed to initialize the Rust implementation"),
                );
            }
        }
    }

    /// `WinFw_InitializeBlocked`.
    pub fn initialize_blocked(
        &mut self,
        settings: &WinFwSettings,
        allowed_endpoint: Option<&EndpointArgument>,
    ) {
        match self {
            Self::Cpp(fw) => fw
                .initialize_blocked(super::TIMEOUT_SECONDS, settings, allowed_endpoint)
                .unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                *manager = Some(
                    winfw::Manager::new_blocked(
                        Duration::from_secs(u64::from(super::TIMEOUT_SECONDS)),
                        &to_settings(settings),
                        allowed_endpoint.map(to_allowed_endpoint).as_ref(),
                    )
                    .expect("Failed to initialize the Rust implementation to a blocking state"),
                );
            }
        }
    }

    /// `WinFw_Deinitialize`.
    pub fn deinitialize(&mut self, cleanup: WinFwCleanupPolicy) {
        match self {
            Self::Cpp(fw) => fw.deinitialize(cleanup).unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                if let Some(mut manager) = manager.take() {
                    manager
                        .deinitialize(to_cleanup_policy(cleanup))
                        .expect("Failed to deinitialize the Rust implementation");
                }
            }
        }
    }

    /// `WinFw_ActivePolicy`.
    pub fn active_policy(&self) -> WinFwActivePolicy {
        match self {
            Self::Cpp(fw) => fw.active_policy().unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => match manager.as_ref().map(winfw::Manager::active_policy) {
                Some(winfw::ActivePolicy::None) => WinFwActivePolicy::None,
                Some(winfw::ActivePolicy::Connecting) => WinFwActivePolicy::Connecting,
                Some(winfw::ActivePolicy::Connected) => WinFwActivePolicy::Connected,
                Some(winfw::ActivePolicy::Blocked) => WinFwActivePolicy::Blocked,
                None => WinFwActivePolicy::None,
            },
        }
    }

    /// `WinFw_ApplyPolicyConnecting`.
    #[expect(clippy::too_many_arguments)]
    pub fn apply_policy_connecting(
        &mut self,
        settings: &WinFwSettings,
        relays: &[EndpointArgument],
        exit_endpoint_ip: Option<&str>,
        relay_clients: &[&str],
        tunnel_interface_alias: Option<&str>,
        allowed_endpoint: &EndpointArgument,
        allowed_tunnel_traffic: AllowedTunnelTraffic<'_>,
    ) -> WinFwPolicyStatus {
        match self {
            Self::Cpp(fw) => fw
                .apply_policy_connecting(
                    settings,
                    relays,
                    exit_endpoint_ip,
                    relay_clients,
                    tunnel_interface_alias,
                    allowed_endpoint,
                    allowed_tunnel_traffic,
                )
                .unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                let Some(manager) = manager.as_mut() else {
                    return WinFwPolicyStatus::GeneralFailure;
                };
                let tunnel = winfw::TunnelParameters {
                    alias: tunnel_interface_alias.map(|alias| alias.to_owned()),
                    exit_ip: exit_endpoint_ip.map(|ip| ip.parse().expect("valid IP")),
                    clients: relay_clients
                        .iter()
                        .map(|client| client.to_string())
                        .collect(),
                    allowed_tunnel_traffic: match allowed_tunnel_traffic.traffic_type {
                        winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::None => {
                            winfw::AllowedTunnelTraffic::None
                        }
                        winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::All => {
                            winfw::AllowedTunnelTraffic::All
                        }
                        winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::One => {
                            winfw::AllowedTunnelTraffic::One(to_relay_endpoint(
                                allowed_tunnel_traffic.endpoint1.expect("endpoint one"),
                            ))
                        }
                        winfw::cpp_ffi::WinFwAllowedTunnelTrafficType::Two => {
                            winfw::AllowedTunnelTraffic::Two(
                                to_relay_endpoint(
                                    allowed_tunnel_traffic.endpoint1.expect("endpoint one"),
                                ),
                                to_relay_endpoint(
                                    allowed_tunnel_traffic.endpoint2.expect("endpoint two"),
                                ),
                            )
                        }
                    },
                };
                let status = manager.apply_policy_connecting(
                    &to_settings(settings),
                    &to_relay_endpoints(relays),
                    &tunnel,
                    &to_allowed_endpoint(allowed_endpoint),
                );
                to_status(status)
            }
        }
    }

    /// `WinFw_ApplyPolicyConnected`.
    #[expect(clippy::too_many_arguments)]
    pub fn apply_policy_connected(
        &mut self,
        settings: &WinFwSettings,
        relays: &[EndpointArgument],
        exit_endpoint_ip: Option<&str>,
        relay_clients: &[&str],
        tunnel_interface_alias: &str,
        tunnel_dns_servers: &[&str],
        non_tunnel_dns_servers: &[&str],
    ) -> WinFwPolicyStatus {
        match self {
            Self::Cpp(fw) => fw
                .apply_policy_connected(
                    settings,
                    relays,
                    exit_endpoint_ip,
                    relay_clients,
                    tunnel_interface_alias,
                    tunnel_dns_servers,
                    non_tunnel_dns_servers,
                )
                .unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                let Some(manager) = manager.as_mut() else {
                    return WinFwPolicyStatus::GeneralFailure;
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
                to_status(status)
            }
        }
    }

    /// `WinFw_ApplyPolicyBlocked`.
    pub fn apply_policy_blocked(
        &mut self,
        settings: &WinFwSettings,
        allowed_endpoint: Option<&EndpointArgument>,
    ) -> WinFwPolicyStatus {
        match self {
            Self::Cpp(fw) => fw.apply_policy_blocked(settings, allowed_endpoint).unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                let Some(manager) = manager.as_mut() else {
                    return WinFwPolicyStatus::GeneralFailure;
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

    /// `WinFw_Reset`.
    pub fn reset(&mut self) -> WinFwPolicyStatus {
        match self {
            Self::Cpp(fw) => fw.reset().unwrap(),
            #[cfg(feature = "rust")]
            Self::Rust(manager) => {
                match manager.as_mut() {
                    Some(manager) => {
                        let result = manager.reset();
                        if let Err(error) = &result {
                            eprintln!("[rust-impl] reset failed: {error:?}");
                        }
                        to_status(result)
                    }
                    // Reset with no initialized module purges everything.
                    None => match winfw::purge_all() {
                        Ok(()) => WinFwPolicyStatus::Success,
                        Err(_) => WinFwPolicyStatus::GeneralFailure,
                    },
                }
            }
        }
    }
}

fn to_settings(settings: &WinFwSettings) -> winfw::Settings {
    winfw::Settings {
        permit_dhcp: settings.permit_dhcp,
        permit_lan: settings.permit_lan,
    }
}

fn to_protocol(protocol: WinFwProtocol) -> winfw::Protocol {
    match protocol {
        WinFwProtocol::Tcp => winfw::Protocol::Tcp,
        WinFwProtocol::Udp => winfw::Protocol::Udp,
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

fn to_cleanup_policy(cleanup: WinFwCleanupPolicy) -> winfw::CleanupPolicy {
    match cleanup {
        WinFwCleanupPolicy::ResetFirewall => winfw::CleanupPolicy::ResetFirewall,
        WinFwCleanupPolicy::BlockingUntilReboot => winfw::CleanupPolicy::BlockingUntilReboot,
    }
}

fn to_status(result: Result<(), winfw::ApplyError>) -> WinFwPolicyStatus {
    if let Err(error) = &result {
        eprintln!("[rust-impl] policy application failed: {error:?}");
    }
    use winfw::ApplyError as RustApplyError;

    match result {
        Ok(()) => WinFwPolicyStatus::Success,
        Err(RustApplyError::LockTimeout(_)) => WinFwPolicyStatus::LockTimeout,
        Err(RustApplyError::Generic(_)) => WinFwPolicyStatus::GeneralFailure,
    }
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
            .deinitialize(WinFwCleanupPolicy::ResetFirewall);
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
/// for tests that drive `WinFw_InitializeBlocked` and the uninitialized
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
