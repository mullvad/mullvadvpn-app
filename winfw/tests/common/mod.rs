//! Shared harness for the WinFW snapshot tests.
//!
//! The dumper in [`wfp_state`] serializes the complete set of WFP objects the
//! firewall owns into a stable, human-readable textual form. Snapshot tests
//! drive the firewall through states, comparing the dumper output at every
//! step against the snapshots in `tests/snapshots/`.

pub mod wfp_state;

use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
    time::Duration,
};

use winfw::{
    ActivePolicy, AllowedEndpoint, ApplyError, CleanupPolicy, Manager, Protocol, RelayEndpoint,
    Settings,
};

/// Timeout for acquiring the WFP transaction lock. Same as the daemon.
const TRANSACTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Serializes all tests: the WFP state the firewall manages is global.
static LOCK: Mutex<()> = Mutex::new(());

/// The firewall under test. Deinitialized with a full reset when dropped, even
/// if a test panics.
pub struct Firewall {
    manager: Option<Manager>,
    _guard: MutexGuard<'static, ()>,
}

impl Firewall {
    fn new() -> Self {
        Self {
            manager: None,
            _guard: LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    pub fn initialize(&mut self) {
        self.manager =
            Some(Manager::new(TRANSACTION_TIMEOUT).expect("Failed to initialize the firewall"));
    }

    pub fn initialize_blocked(
        &mut self,
        settings: &Settings,
        allowed_endpoint: Option<&AllowedEndpoint>,
    ) {
        self.manager = Some(
            Manager::new_blocked(TRANSACTION_TIMEOUT, settings, allowed_endpoint)
                .expect("Failed to initialize the firewall to a blocking state"),
        );
    }

    pub fn deinitialize(&mut self, cleanup: CleanupPolicy) {
        if let Some(mut manager) = self.manager.take() {
            manager
                .deinitialize(cleanup)
                .expect("Failed to deinitialize the firewall");
        }
    }

    /// The initialized firewall.
    pub fn manager(&mut self) -> &mut Manager {
        self.manager
            .as_mut()
            .expect("The firewall is not initialized")
    }

    pub fn active_policy(&self) -> ActivePolicy {
        self.manager
            .as_ref()
            .map_or(ActivePolicy::None, Manager::active_policy)
    }

    /// Reset the policy in effect or, without an initialized firewall, remove
    /// everything it ever registered.
    pub fn reset(&mut self) -> Result<(), ApplyError> {
        match self.manager.as_mut() {
            Some(manager) => manager.reset(),
            None => winfw::purge_all(),
        }
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        self.deinitialize(CleanupPolicy::ResetFirewall);
    }
}

/// Run a test body against an initialized firewall.
pub fn run_test(body: impl FnOnce(&mut Firewall)) {
    let mut firewall = Firewall::new();
    firewall.initialize();
    body(&mut firewall);
}

/// Run a test body against an uninitialized firewall, for tests that drive
/// the blocked initialization and the uninitialized reset themselves.
pub fn run_test_uninitialized(body: impl FnOnce(&mut Firewall)) {
    body(&mut Firewall::new());
}

pub fn endpoint(ip: &str, port: u16, protocol: Protocol) -> RelayEndpoint {
    RelayEndpoint {
        ip: ip.parse().expect("valid IP"),
        port,
        protocol,
    }
}

pub fn allowed_endpoint(
    ip: &str,
    port: u16,
    protocol: Protocol,
    clients: &[&str],
) -> AllowedEndpoint {
    AllowedEndpoint {
        clients: clients.iter().map(PathBuf::from).collect(),
        endpoint: endpoint(ip, port, protocol),
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
