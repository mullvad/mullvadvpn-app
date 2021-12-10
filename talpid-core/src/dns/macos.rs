use crate::tunnel_state_machine::TunnelCommand;
use futures::channel::mpsc;
use log::{debug, trace};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fmt,
    net::{AddrParseError, IpAddr},
    sync::{mpsc as sync_mpsc, Arc},
    thread,
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

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen when setting/monitoring DNS on macOS.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Error while setting DNS servers
    #[error(display = "Error while setting DNS servers")]
    SettingDnsFailed,

    /// Failed to initialize dynamic store
    #[error(display = "Failed to initialize dynamic store")]
    DynamicStoreInitError,

    /// Failed to parse IP address from config string
    #[error(display = "Failed to parse an IP address from a config string")]
    AddrParseError(String, String, AddrParseError),

    /// Failed to obtain name for interface
    #[error(display = "Failed to obtain interface name")]
    GetInterfaceNameError,

    /// Failed to load interface config
    #[error(display = "Failed to load interface config at path {}", _0)]
    LoadInterfaceConfigError(String),

    /// Failed to load DNS config
    #[error(display = "Failed to load DNS config at path {}", _0)]
    LoadDnsConfigError(String),
}

const STATE_PATH_PATTERN: &str = "State:/Network/Service/.*/DNS";
const SETUP_PATH_PATTERN: &str = "Setup:/Network/Service/.*/DNS";

type ServicePath = String;
type DnsServer = String;

struct State {
    /// The settings this monitor is currently enforcing as active settings.
    dns_settings: DnsSettings,
    /// The backup of all DNS settings. These are being applied back on reset.
    backup: HashMap<ServicePath, Option<DnsSettings>>,
    /// Tunnel command sender for reporting updates to the system DNS config
    tunnel_tx: std::sync::Weak<mpsc::UnboundedSender<crate::tunnel_state_machine::TunnelCommand>>,
}

impl State {
    fn send_new_config(&self) {
        if let Some(tunnel_tx) = self.tunnel_tx.upgrade() {
            match parse_sc_config(&self.backup) {
                Ok(config) => {
                    // TODO: do better filtering to get the best resolver
                    let _ = tunnel_tx
                        .unbounded_send(TunnelCommand::HostDnsConfig(config.into_iter().next()));
                }
                Err(err) => {
                    log::error!("Failed to parse host's DNS config: {}", err);
                }
            };
        }
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
        trace!(
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
            .unwrap_or(Vec::new())
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
    state: Arc<Mutex<Option<State>>>,

    tunnel_tx: std::sync::Weak<mpsc::UnboundedSender<crate::tunnel_state_machine::TunnelCommand>>,
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
    fn new(tunnel_tx: std::sync::Weak<mpsc::UnboundedSender<TunnelCommand>>) -> Result<Self> {
        let state = Arc::new(Mutex::new(None));
        Self::spawn(state.clone())?;
        Ok(DnsMonitor {
            store: SCDynamicStoreBuilder::new("mullvad-dns").build(),
            state,
            tunnel_tx,
        })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        let servers: Vec<DnsServer> = servers.iter().map(|ip| ip.to_string()).collect();
        let settings = DnsSettings::from_server_addresses(&servers, interface.to_string());
        let mut state_lock = self.state.lock();
        *state_lock = Some(match state_lock.take() {
            None => {
                let backup = read_all_dns(&self.store);
                trace!("Backup of DNS settings: {:#?}", backup);
                for service_path in backup.keys() {
                    settings.save(&self.store, service_path.as_str())?;
                }
                State {
                    dns_settings: settings,
                    backup,
                    tunnel_tx: self.tunnel_tx.clone(),
                }
            }
            Some(state) => {
                if servers != state.dns_settings.server_addresses() {
                    for service_path in state.backup.keys() {
                        settings.save(&self.store, service_path.as_str())?;
                    }
                    State {
                        dns_settings: settings,
                        backup: state.backup,
                        tunnel_tx: self.tunnel_tx.clone(),
                    }
                } else {
                    debug!("No change, new DNS same as the one already set");
                    state
                }
            }
        });
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        let mut state_lock = self.state.lock();
        if let Some(state) = state_lock.take() {
            trace!("Restoring DNS settings to: {:#?}", state.backup);
            for (service_path, settings) in state.backup {
                if let Some(settings) = settings {
                    settings.save(&self.store, service_path.as_str())?;
                } else {
                    debug!("Removing DNS for {}", service_path);
                    if !self.store.remove(CFString::new(&service_path)) {
                        return Err(Error::SettingDnsFailed);
                    }
                }
            }
        }
        Ok(())
    }
}

impl DnsMonitor {
    /// Spawns the background thread running the CoreFoundation main loop and monitors the system
    /// for DNS changes.
    fn spawn(state: Arc<Mutex<Option<State>>>) -> Result<()> {
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
        self.state
            .lock()
            .as_ref()
            .map(|state| parse_sc_config(&state.backup))
            .unwrap_or_else(|| parse_sc_config(&read_all_dns(&self.store)))
    }
}

fn parse_sc_config(
    config: &HashMap<String, Option<DnsSettings>>,
) -> Result<Option<(String, Vec<IpAddr>)>> {
    config
        .iter()
        .filter_map(|(path, maybe_config)| {
            if let Some(settings) = maybe_config {
                Some((path, settings))
            } else {
                None
            }
        })
        .map(|(path, settings)| {
            let addresses = settings.interface_config(path.as_str())?;
            Ok((settings.name.clone(), addresses))
        })
        .next()
        .transpose()
}

/// Creates a `SCDynamicStore` that watches all network interfaces for changes to the DNS settings.
fn create_dynamic_store(state: Arc<Mutex<Option<State>>>) -> Result<SCDynamicStore> {
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
        trace!("Registered for dynamic store notifications");
        Ok(store)
    } else {
        Err(Error::DynamicStoreInitError)
    }
}

fn run_dynamic_store_runloop(store: SCDynamicStore) {
    let run_loop_source = store.create_run_loop_source();
    CFRunLoop::get_current().add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });

    trace!("Entering DNS CFRunLoop");
    CFRunLoop::run_current();
}

/// This function is called by the Core Foundation event loop when there is a change to one or more
/// watched dynamic store values. In our case we watch all DNS settings.
fn dns_change_callback(
    store: SCDynamicStore,
    changed_keys: CFArray<CFString>,
    state: &mut Arc<Mutex<Option<State>>>,
) {
    let mut state_lock = state.lock();
    match *state_lock {
        None => {
            trace!("Not injecting DNS at this time");
        }
        Some(ref mut state) => {
            dns_change_callback_internal(store, changed_keys, state);
        }
    }
}

fn dns_change_callback_internal(
    store: SCDynamicStore,
    changed_keys: CFArray<CFString>,
    state: &mut State,
) {
    state.send_new_config();
    for path in &changed_keys {
        let should_set_dns = match DnsSettings::load(&store, path.clone()).ok() {
            None => {
                debug!("Detected DNS removed for {}", *path);
                state.backup.insert(path.to_string(), None);
                true
            }
            Some(new_settings) => {
                if new_settings.dict != state.dns_settings.dict {
                    debug!("Detected DNS change for {}", *path);
                    state.backup.insert(path.to_string(), Some(new_settings));
                    true
                } else {
                    trace!("Ignoring DNS change since it's equal to desired DNS");
                    false
                }
            }
        };
        if should_set_dns {
            if let Err(e) = state.dns_settings.save(&store, path.clone()) {
                log::error!("Failed changing DNS for {}: {}", *path, e);
            }
            // If we changed a "state" entry, also set the corresponding "setup" entry.
            if let Some(setup_path_str) = state_to_setup_path(&path.to_string()) {
                let setup_path = CFString::new(&setup_path_str);
                if !state.backup.contains_key(&setup_path_str) {
                    state.backup.insert(
                        setup_path_str,
                        DnsSettings::load(&store, setup_path.clone()).ok(),
                    );
                }
                if let Err(e) = state.dns_settings.save(&store, setup_path.clone()) {
                    log::error!("Failed changing DNS for {}: {}", setup_path, e);
                }
            }
        }
    }
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
            if !backup.contains_key(&setup_path_str) {
                backup.insert(
                    setup_path_str,
                    DnsSettings::load(store, setup_path.clone()).ok(),
                );
            }
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
