//! Shared harness for the WinFW snapshot tests.
//!
//! The dumper in [`wfp_state`] serializes the complete set of WFP objects the
//! firewall owns into a stable, human-readable textual form. Snapshot tests
//! drive the firewall implementation through states, and compare the dumper
//! output at every step against the snapshots in `tests/snapshots/`.
//!
//! Run with `cargo test` as administrator (graphical session not required).
//! Regenerate with `INSTA_UPDATE=always cargo test -p winfw`, or use
//! `cargo insta test` and review the pending snapshots with
//! `cargo insta review`. Set `WINFW_DLL_PATH` to the C++ `winfw.dll` to use
//! the C++ reference implementation.

pub mod wfp_state;

use std::sync::OnceLock;

use winfw::cpp_ffi::{
    AllowedTunnelTraffic, CppWinFw, EndpointArgument, WinFwActivePolicy, WinFwCleanupPolicy,
    WinFwPolicyStatus, WinFwSettings,
};

/// The firewall implementation under test.
pub enum Implementation {
    /// The C++ WinFW library, loaded from `winfw.dll`.
    Cpp(CppWinFw),
}

impl Implementation {
    /// Create the implementation selected by cargo features.
    pub fn load() -> Self {
        #[cfg(feature = "cpp-ffi")]
        {
            Self::Cpp(CppWinFw::load_default().expect("Failed to load winfw.dll"))
        }
        #[cfg(not(feature = "cpp-ffi"))]
        {
            compile_error!("Select an implementation feature, e.g. --features cpp-ffi")
        }
    }
}

impl Implementation {
    /// `WinFw_Initialize`.
    pub fn initialize(&mut self) {
        match self {
            Self::Cpp(fw) => fw.initialize(super::TIMEOUT_SECONDS).unwrap(),
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
        }
    }

    /// `WinFw_Deinitialize`.
    pub fn deinitialize(&mut self, cleanup: WinFwCleanupPolicy) {
        match self {
            Self::Cpp(fw) => fw.deinitialize(cleanup).unwrap(),
        }
    }

    /// `WinFw_ActivePolicy`.
    pub fn active_policy(&self) -> WinFwActivePolicy {
        match self {
            Self::Cpp(fw) => fw.active_policy().unwrap(),
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
        }
    }

    /// `WinFw_Reset`.
    pub fn reset(&mut self) -> WinFwPolicyStatus {
        match self {
            Self::Cpp(fw) => fw.reset().unwrap(),
        }
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

    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211, IP_ADAPTER_ADDRESSES_LH,
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
