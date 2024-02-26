use futures::channel::mpsc;
use parking_lot::Mutex;
use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    net::{AddrParseError, IpAddr},
    sync::{mpsc as sync_mpsc, Arc, Weak},
    thread,
    time::Duration,
};
use system_configuration::{
    core_foundation::{
        array::CFArray,
        base::{CFType, TCFType, ToVoid},
        dictionary::{CFDictionary, CFMutableDictionary},
        propertylist::CFPropertyList,
        runloop::{kCFRunLoopCommonModes, CFRunLoop},
        string::CFString,
    },
    dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder, SCDynamicStoreCallBackContext},
    sys::schema_definitions::{kSCPropNetDNSServerAddresses, kSCPropNetInterfaceDeviceName},
};
use talpid_time::Instant;
use talpid_types::tunnel::ErrorStateCause;

use crate::tunnel_state_machine::TunnelCommand;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen when setting/monitoring DNS on macOS.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error while setting DNS servers
    #[error("Error while setting DNS servers")]
    SettingDnsFailed,

    /// Failed to initialize dynamic store
    #[error("Failed to initialize dynamic store")]
    DynamicStoreInitError,

    /// Failed to parse IP address from config string
    #[error("Failed to parse an IP address from a config string")]
    AddrParseError(String, String, AddrParseError),

    /// Failed to obtain name for interface
    #[error("Failed to obtain interface name")]
    GetInterfaceNameError,

    /// Failed to load interface config
    #[error("Failed to load interface config at path {0}")]
    LoadInterfaceConfigError(String),

    /// Failed to load DNS config
    #[error("Failed to load DNS config at path {0}")]
    LoadDnsConfigError(String),
}

const STATE_PATH_PATTERN: &str = "State:/Network/Service/.*/DNS";
const SETUP_PATH_PATTERN: &str = "Setup:/Network/Service/.*/DNS";

type ServicePath = String;
type DnsServer = String;

struct State {
    /// Channel to signal to the TSM that something has gone wrong
    tsm_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
    /// Change counter to fail a tunnel if setting DNS
    change_counter: ChangeCounter,
    /// The settings this monitor is currently enforcing as active settings.
    dns_settings: Option<DnsSettings>,
    /// The backup of all DNS settings. These are being applied back on reset.
    backup: HashMap<ServicePath, Option<DnsSettings>>,
}

impl State {
    fn new(tsm_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>) -> Self {
        Self {
            tsm_tx,
            dns_settings: None,
            change_counter: ChangeCounter::new(),
            backup: HashMap::new(),
        }
    }

    fn apply_new_config(
        &mut self,
        store: &SCDynamicStore,
        interface: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        let servers: Vec<DnsServer> = servers.iter().map(|ip| ip.to_string()).collect();
        let new_settings = DnsSettings::from_server_addresses(&servers, interface.to_string());
        match &self.dns_settings {
            None => {
                let backup = read_all_dns(store);
                log::trace!("Backup of DNS settings: {:#?}", backup);
                for service_path in backup.keys() {
                    new_settings.save(store, service_path.as_str())?;
                }
                self.dns_settings = Some(new_settings);
                self.backup = backup;
            }
            Some(old_settings) => {
                if new_settings.address_set() != old_settings.address_set() {
                    for service_path in self.backup.keys() {
                        new_settings.save(store, service_path.as_str())?;
                    }
                    self.dns_settings = Some(new_settings);
                }
            }
        };
        self.change_counter.clear();

        Ok(())
    }

    fn on_changed_keys(&mut self, store: SCDynamicStore, changed_keys: CFArray<CFString>) {
        if let Some(expected_settings) = &self.dns_settings {
            for path in &changed_keys {
                let should_set_dns = match DnsSettings::load(&store, path.clone()).ok() {
                    None => {
                        log::debug!("Detected DNS removed for {}", *path);
                        self.backup.insert(path.to_string(), None);
                        true
                    }
                    Some(new_settings) => {
                        if new_settings.address_set() != expected_settings.address_set() {
                            let servers = new_settings.server_addresses().join(",");
                            log::debug!("Detected DNS change [{}] for {}", servers, *path);
                            self.backup.insert(path.to_string(), Some(new_settings));
                            true
                        } else {
                            log::trace!("Ignoring DNS change since it's equal to desired DNS");
                            false
                        }
                    }
                };
                if should_set_dns {
                    if self.change_counter.increment() {
                        if let Some(tx) = self.tsm_tx.upgrade() {
                            log::error!("A burst of DNS changes has been detected, assuming can't set DNS config properly");
                            let _ = tx
                                .unbounded_send(TunnelCommand::Block(ErrorStateCause::SetDnsError));
                        }

                        if let Err(err) = self.reset(&store) {
                            log::error!("Failed to reset DNS after detecting a burst: {}", err);
                        }
                        return;
                    }
                    if let Err(e) = expected_settings.save(&store, path.clone()) {
                        log::error!("Failed changing DNS for {}: {}", *path, e);
                    }
                    // If we changed a "state" entry, also set the corresponding "setup" entry.
                    if let Some(setup_path_str) = state_to_setup_path(&path.to_string()) {
                        let setup_path = CFString::new(&setup_path_str);
                        self.backup
                            .entry(setup_path_str)
                            .or_insert_with(|| DnsSettings::load(&store, setup_path.clone()).ok());
                        if let Err(e) = expected_settings.save(&store, setup_path.clone()) {
                            log::error!("Failed changing DNS for {}: {}", setup_path, e);
                        }
                    }
                }
            }
        }
    }

    fn reset(&mut self, store: &SCDynamicStore) -> Result<()> {
        log::trace!("Restoring DNS settings to: {:#?}", self.backup);
        let old_backup = std::mem::take(&mut self.backup);
        self.dns_settings.take();
        for (service_path, settings) in old_backup {
            if let Some(settings) = settings {
                settings.save(store, service_path.as_str())?;
            } else {
                log::debug!("Removing DNS for {}", service_path);
                if !store.remove(CFString::new(&service_path)) {
                    return Err(Error::SettingDnsFailed);
                }
            }
        }
        Ok(())
    }
}

/// Holds the configuration for one service.
#[derive(Debug, Eq, PartialEq, Clone)]
struct DnsSettings {
    dict: CFDictionary,
    name: String,
}

unsafe impl Send for DnsSettings {}

impl DnsSettings {
    pub fn from_server_addresses(server_addresses: &[DnsServer], name: String) -> Self {
        let mut mut_dict = CFMutableDictionary::new();
        if !server_addresses.is_empty() {
            let cf_string_servers: Vec<CFString> =
                server_addresses.iter().map(|s| CFString::new(s)).collect();
            let server_addresses_value = CFArray::from_CFTypes(&cf_string_servers).into_untyped();
            let server_addresses_key =
                unsafe { CFString::wrap_under_get_rule(kSCPropNetDNSServerAddresses) };
            mut_dict.add(
                &server_addresses_key.to_void(),
                &server_addresses_value.to_void(),
            );
        }
        let dict = mut_dict.to_immutable();
        DnsSettings { dict, name }
    }

    /// Get DNS settings for a given service path. Returns `None` If the path does not exist.
    pub fn load<S: Into<CFString>>(store: &SCDynamicStore, path: S) -> Result<Self> {
        let cf_path = path.into();

        let dict = store
            .get(cf_path.clone())
            .and_then(CFPropertyList::downcast_into::<CFDictionary>)
            .ok_or(Error::LoadDnsConfigError(cf_path.to_string()))?;

        let name =
            InterfaceSettings::load_from_dns_key(store, cf_path.to_string())?.interface_name()?;

        Ok(DnsSettings { dict, name })
    }

    /// Set the dynamic store entry at `path` to a dictionary these DNS settings.
    pub fn save<S: Into<CFString> + fmt::Display>(
        &self,
        store: &SCDynamicStore,
        path: S,
    ) -> Result<()> {
        log::trace!(
            "Setting DNS to [{}] for {}",
            self.server_addresses().join(", "),
            path.to_string()
        );
        if store.set(path, self.dict.clone()) {
            Ok(())
        } else {
            Err(Error::SettingDnsFailed)
        }
    }

    pub fn server_addresses(&self) -> Vec<String> {
        self.dict
            .find(unsafe { kSCPropNetDNSServerAddresses }.to_void())
            .map(|array_ptr| unsafe { CFType::wrap_under_get_rule(*array_ptr) })
            .and_then(|array| array.downcast::<CFArray>())
            .and_then(Self::parse_cf_array_to_strings)
            .unwrap_or_default()
    }

    pub fn address_set(&self) -> BTreeSet<String> {
        BTreeSet::from_iter(self.server_addresses())
    }

    pub fn interface_config(&self, interface_path: &str) -> Result<Vec<IpAddr>> {
        let addresses = self
            .server_addresses()
            .into_iter()
            .map(|server_addr| {
                server_addr.parse().map_err(|err| {
                    Error::AddrParseError(interface_path.to_string(), server_addr.clone(), err)
                })
            })
            .collect::<Result<Vec<IpAddr>>>()?;

        Ok(addresses)
    }

    /// Parses a CFArray into a Rust vector of Rust strings, if the array contains CFString
    /// instances only, otherwise `None` is returned.
    fn parse_cf_array_to_strings(array: CFArray) -> Option<Vec<String>> {
        let mut strings = Vec::new();
        for item_ptr in array.iter() {
            let item = unsafe { CFType::wrap_under_get_rule(*item_ptr) };
            if let Some(string) = item.downcast::<CFString>() {
                strings.push(string.to_string());
            } else {
                log::error!("DNS server entry is not a string: {:?}", item);
                return None;
            };
        }
        Some(strings)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct InterfaceSettings(CFDictionary);

impl InterfaceSettings {
    /// Get network interface settings for the given path
    pub fn load_from_dns_key(store: &SCDynamicStore, dns_path: String) -> Result<Self> {
        // remove the "DNS" part of the path
        let path = match dns_path.strip_prefix("State") {
            Some(service_path) => "Setup".to_owned() + service_path,
            None => dns_path.to_string(),
        };
        let interface_path = path.replace("/DNS", "/Interface");

        Ok(Self(
            store
                .get(CFString::from(interface_path.as_str()))
                .and_then(CFPropertyList::downcast_into::<CFDictionary>)
                .ok_or(Error::LoadInterfaceConfigError(path))?,
        ))
    }

    pub fn interface_name(&self) -> Result<String> {
        self.0
            .find(unsafe { kSCPropNetInterfaceDeviceName }.to_void())
            .map(|str_pointer| unsafe { CFType::wrap_under_get_rule(*str_pointer) })
            .and_then(|string| string.downcast::<CFString>())
            .map(|cf_string| cf_string.to_string())
            .ok_or(Error::GetInterfaceNameError)
    }
}

unsafe impl Send for InterfaceSettings {}

pub struct DnsMonitor {
    store: SCDynamicStore,

    /// The current DNS injection state. If this is `None` it means we are not injecting any DNS.
    /// When it's `Some(state)` we are actively making sure `state.dns_settings` is configured
    /// on all network interfaces.
    state: Arc<Mutex<State>>,
}

/// SAFETY: The `SCDynamicStore` can be sent to other threads since it doesn't share mutable state
/// with anything else.
unsafe impl Send for DnsMonitor {}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    /// Creates and returns a new `DnsMonitor`. This spawns a background thread that will monitor
    /// DNS settings for all network interfaces. If any changes occur it will instantly reset
    /// the DNS settings for that interface back to the last server list set to this instance
    /// with `set_dns`.
    fn new(tx: Weak<mpsc::UnboundedSender<TunnelCommand>>) -> Result<Self> {
        let state = Arc::new(Mutex::new(State::new(tx)));
        Self::spawn(state.clone())?;
        Ok(DnsMonitor {
            store: SCDynamicStoreBuilder::new("mullvad-dns").build(),
            state,
        })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        let mut state = self.state.lock();
        state.apply_new_config(&self.store, interface, servers)
    }

    fn reset(&mut self) -> Result<()> {
        self.state.lock().reset(&self.store)
    }
}

impl DnsMonitor {
    /// Spawns the background thread running the CoreFoundation main loop and monitors the system
    /// for DNS changes.
    fn spawn(state: Arc<Mutex<State>>) -> Result<()> {
        let (result_tx, result_rx) = sync_mpsc::channel();
        thread::spawn(move || match create_dynamic_store(state) {
            Ok(store) => {
                result_tx.send(Ok(())).unwrap();
                run_dynamic_store_runloop(store);
                // TODO(linus): This is critical. Improve later by sending error signal to Daemon
                log::error!("Core Foundation main loop exited! It should run forever");
            }
            Err(e) => result_tx.send(Err(e)).unwrap(),
        });
        result_rx.recv().unwrap()
    }
    /// Get the system config without our changes
    pub fn get_system_config(&self) -> Result<Option<(String, Vec<IpAddr>)>> {
        let state = self.state.lock();
        if state.dns_settings.is_some() {
            parse_sc_config(&state.backup)
        } else {
            parse_sc_config(&read_all_dns(&self.store))
        }
    }
}

fn parse_sc_config(
    config: &HashMap<String, Option<DnsSettings>>,
) -> Result<Option<(String, Vec<IpAddr>)>> {
    config
        .iter()
        .filter_map(|(path, maybe_config)| maybe_config.as_ref().map(|settings| (path, settings)))
        .map(|(path, settings)| {
            let addresses = settings.interface_config(path.as_str())?;
            Ok((settings.name.clone(), addresses))
        })
        .next()
        .transpose()
}

/// Creates a `SCDynamicStore` that watches all network interfaces for changes to the DNS settings.
fn create_dynamic_store(state: Arc<Mutex<State>>) -> Result<SCDynamicStore> {
    let callback_context = SCDynamicStoreCallBackContext {
        callout: dns_change_callback,
        info: state,
    };

    let store = SCDynamicStoreBuilder::new("talpid-dns-monitor")
        .callback_context(callback_context)
        .build();

    let watch_keys: CFArray<CFString> = CFArray::from_CFTypes(&[]);
    let watch_patterns = CFArray::from_CFTypes(&[
        CFString::new(STATE_PATH_PATTERN),
        CFString::new(SETUP_PATH_PATTERN),
    ]);

    if store.set_notification_keys(&watch_keys, &watch_patterns) {
        log::trace!("Registered for dynamic store notifications");
        Ok(store)
    } else {
        Err(Error::DynamicStoreInitError)
    }
}

fn run_dynamic_store_runloop(store: SCDynamicStore) {
    let run_loop_source = store.create_run_loop_source();
    CFRunLoop::get_current().add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });

    log::trace!("Entering DNS CFRunLoop");
    CFRunLoop::run_current();
}

/// This function is called by the Core Foundation event loop when there is a change to one or more
/// watched dynamic store values. In our case we watch all DNS settings.
fn dns_change_callback(
    store: SCDynamicStore,
    changed_keys: CFArray<CFString>,
    state: &mut Arc<Mutex<State>>,
) {
    state.lock().on_changed_keys(store, changed_keys)
}

/// Read all existing DNS settings and return them.
fn read_all_dns(store: &SCDynamicStore) -> HashMap<ServicePath, Option<DnsSettings>> {
    let mut backup = HashMap::new();
    // Backup all "state" DNS, and all corresponding "setup" DNS even if they don't exist
    if let Some(paths) = store.get_keys(STATE_PATH_PATTERN) {
        for state_path in paths.iter() {
            let state_path_str = state_path.to_string();
            let setup_path_str = state_to_setup_path(&state_path_str).unwrap();
            backup.insert(
                state_path_str,
                DnsSettings::load(store, state_path.clone()).ok(),
            );
            backup.insert(
                setup_path_str.clone(),
                DnsSettings::load(store, setup_path_str.as_ref()).ok(),
            );
        }
    }
    // Backup all "setup" DNS not already covered
    if let Some(paths) = store.get_keys(SETUP_PATH_PATTERN) {
        for setup_path in paths.iter() {
            let setup_path_str = setup_path.to_string();
            backup
                .entry(setup_path_str)
                .or_insert_with(|| DnsSettings::load(store, setup_path.clone()).ok());
        }
    }
    backup
}

fn state_to_setup_path(state_path: &str) -> Option<String> {
    if state_path.starts_with("State:/") {
        Some(state_path.replacen("State:/", "Setup:/", 1))
    } else {
        None
    }
}

const MAX_CHANGES_PER_INTERVAL: usize = 25;
const FIVE_SECONDS: Duration = Duration::from_secs(5);

/// Effectively a circular buffer of `Instant`s of when was the last time a DNS change occurred.
struct ChangeCounter {
    changes: Vec<Instant>,
}

impl ChangeCounter {
    fn new() -> Self {
        Self {
            changes: Vec::with_capacity(MAX_CHANGES_PER_INTERVAL),
        }
    }

    fn clear(&mut self) {
        self.changes.clear();
    }

    fn increment(&mut self) -> bool {
        let now = Instant::now();
        self.changes
            .retain(|old_change| now.duration_since(*old_change) < FIVE_SECONDS);
        self.changes.push(now);
        self.changes.len() >= MAX_CHANGES_PER_INTERVAL
    }
}
