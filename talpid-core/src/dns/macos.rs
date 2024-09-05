use parking_lot::Mutex;
use std::{
    collections::{BTreeSet, HashMap},
    fmt, mem,
    net::IpAddr,
    sync::{mpsc as sync_mpsc, Arc, RwLock},
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
use talpid_routing::debounce::BurstGuard;

use super::ResolvedDnsConfig;

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

const BURST_BUFFER_PERIOD: Duration = Duration::from_millis(500);
const BURST_LONGEST_BUFFER_PERIOD: Duration = Duration::from_secs(5);

type ServicePath = String;
type DnsServer = String;

struct State {
    /// The settings this monitor is currently enforcing as active settings.
    dns_settings: Option<DnsSettings>,
    /// The backup of all DNS settings. These are being applied back on reset.
    backup: HashMap<ServicePath, Option<DnsSettings>>,
}

impl State {
    fn new() -> Self {
        Self {
            dns_settings: None,
            backup: HashMap::new(),
        }
    }

    fn apply_new_config(
        &mut self,
        store: &SCDynamicStore,
        interface: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        talpid_types::detect_flood!();

        let servers: Vec<DnsServer> = servers.iter().map(|ip| ip.to_string()).collect();
        let new_settings = DnsSettings::from_server_addresses(&servers, interface.to_string());
        match &self.dns_settings {
            None => {
                self.dns_settings = Some(new_settings);
                self.update_and_apply_state(store);
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

        Ok(())
    }

    /// Store changes to the DNS config, ignoring any changes that we have applied. Then apply our
    /// desired state to any services to which it has not already been applied.
    fn update_and_apply_state(&mut self, store: &SCDynamicStore) {
        let actual_state = read_all_dns(store);
        self.update_backup_state(&actual_state);
        self.apply_desired_state(store, &actual_state);
    }

    /// Store changes to the DNS config, ignoring any changes that we have applied. The operation is
    /// idempotent.
    fn update_backup_state(&mut self, actual_state: &HashMap<ServicePath, Option<DnsSettings>>) {
        let Some(ref desired_settings) = self.dns_settings else {
            return;
        };

        let prev_state = mem::take(&mut self.backup);
        let desired_set = desired_settings.address_set();

        self.backup = Self::merge_states(actual_state, prev_state, desired_set);
    }

    /// Merge `new_state` set by the OS with a previous `prev_state`, but ignore any service whose
    /// addresses are `ignore_addresses`.
    fn merge_states(
        new_state: &HashMap<ServicePath, Option<DnsSettings>>,
        mut prev_state: HashMap<ServicePath, Option<DnsSettings>>,
        ignore_addresses: BTreeSet<String>,
    ) -> HashMap<ServicePath, Option<DnsSettings>> {
        let mut modified_state = HashMap::new();

        for (path, settings) in new_state {
            let old_entry = prev_state.remove(path);
            match settings {
                // If the service is using the desired addresses, don't save changes
                Some(settings) if settings.address_set() == ignore_addresses => {
                    let settings = old_entry.unwrap_or_else(|| Some(settings.to_owned()));
                    modified_state.insert(path.to_owned(), settings);
                }
                // Otherwise, save the new settings
                settings => {
                    let servers = settings
                        .as_ref()
                        .map(|settings| settings.server_addresses().join(","))
                        .unwrap_or_default();
                    log::debug!("Saving DNS settings [{}] for {}", servers, path);
                    modified_state.insert(path.to_owned(), settings.to_owned());
                }
            }
        }

        for path in prev_state.keys() {
            log::debug!("DNS removed for {path}");
        }

        modified_state
    }

    /// Apply the desired addresses to all network services. The operation is idempotent.
    fn apply_desired_state(
        &mut self,
        store: &SCDynamicStore,
        actual_state: &HashMap<ServicePath, Option<DnsSettings>>,
    ) {
        let Some(ref desired_settings) = self.dns_settings else {
            return;
        };
        let desired_set = desired_settings.address_set();

        for (path, settings) in actual_state {
            match settings {
                // Do nothing if the state is already what we want
                Some(settings) if settings.address_set() == desired_set => (),
                // Apply desired state to service
                _ => {
                    let path_cf = CFString::new(path);
                    if let Err(e) = desired_settings.save(store, path_cf) {
                        log::error!("Failed changing DNS for {}: {}", path, e);
                    }
                }
            }
        }
    }

    fn reset(&mut self, store: &SCDynamicStore) -> Result<()> {
        log::trace!("Restoring DNS settings to: {:#?}", self.backup);

        let actual_state = read_all_dns(store);
        self.update_backup_state(&actual_state);
        self.dns_settings.take();

        let old_backup = std::mem::take(&mut self.backup);

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
    fn new() -> Result<Self> {
        let state = Arc::new(Mutex::new(State::new()));
        Self::spawn(state.clone())?;
        Ok(DnsMonitor {
            store: SCDynamicStoreBuilder::new("mullvad-dns").build(),
            state,
        })
    }

    fn set(&mut self, interface: &str, config: ResolvedDnsConfig) -> Result<()> {
        let servers = config.addresses().to_owned();

        let mut state = self.state.lock();
        state.apply_new_config(&self.store, interface, &servers)
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
}

/// Creates a `SCDynamicStore` that watches all network interfaces for changes to the DNS settings.
fn create_dynamic_store(state: Arc<Mutex<State>>) -> Result<SCDynamicStore> {
    struct StoreContainer {
        store: SCDynamicStore,
    }
    // SAFETY: The store is thread-safe
    unsafe impl Send for StoreContainer {}
    // SAFETY: The store is thread-safe
    unsafe impl Sync for StoreContainer {}

    let store_container: Arc<RwLock<Option<StoreContainer>>> = Arc::new(RwLock::new(None));
    let store_container_copy = store_container.clone();

    let update_trigger = BurstGuard::new(
        BURST_BUFFER_PERIOD,
        BURST_LONGEST_BUFFER_PERIOD,
        move || {
            if let Some(store) = &*store_container.read().unwrap() {
                state.lock().update_and_apply_state(&store.store);
            }
        },
    );

    let callback_context = SCDynamicStoreCallBackContext {
        callout: dns_change_callback,
        info: update_trigger,
    };

    let store = SCDynamicStoreBuilder::new("talpid-dns-monitor")
        .callback_context(callback_context)
        .build();

    let mut store_container = store_container_copy.write().unwrap();
    *store_container = Some(StoreContainer {
        store: store.clone(),
    });

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
    _store: SCDynamicStore,
    _changed_keys: CFArray<CFString>,
    state: &mut BurstGuard,
) {
    state.trigger();
}

/// Read all existing DNS settings and return them.
fn read_all_dns(store: &SCDynamicStore) -> HashMap<ServicePath, Option<DnsSettings>> {
    let mut settings: HashMap<_, _> = HashMap::new();
    // All "state" DNS, and all corresponding "setup" DNS even if they don't exist
    if let Some(paths) = store.get_keys(STATE_PATH_PATTERN) {
        for state_path in paths.iter() {
            let state_path_str = state_path.to_string();
            let setup_path_str = state_to_setup_path(&state_path_str).unwrap();
            settings.insert(
                state_path_str,
                DnsSettings::load(store, state_path.clone()).ok(),
            );
            settings.insert(
                setup_path_str.clone(),
                DnsSettings::load(store, setup_path_str.as_ref()).ok(),
            );
        }
    }
    // All "setup" DNS not already covered
    if let Some(paths) = store.get_keys(SETUP_PATH_PATTERN) {
        for setup_path in paths.iter() {
            let setup_path_str = setup_path.to_string();
            settings
                .entry(setup_path_str)
                .or_insert_with(|| DnsSettings::load(store, setup_path.clone()).ok());
        }
    }
    settings
}

fn state_to_setup_path(state_path: &str) -> Option<String> {
    if state_path.starts_with("State:/") {
        Some(state_path.replacen("State:/", "Setup:/", 1))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::{DnsSettings, State};
    use std::collections::{BTreeSet, HashMap};

    /// The initial backup should equal whatever the first provided state is.
    #[test]
    fn test_backup_new_dns_config() {
        let prev_state = HashMap::new();

        let new_state = HashMap::from([
            ("a".to_owned(), None),
            (
                "b".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["1.2.3.4".to_owned()],
                    "iface_b".to_owned(),
                )),
            ),
            // One of our states already equals the desired state. It should be stored regardless.
            (
                "c".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["10.64.0.1".to_owned()],
                    "iface_c".to_owned(),
                )),
            ),
        ]);

        let desired_addresses: BTreeSet<String> = ["10.64.0.1".to_owned()].into();

        let merged_state = State::merge_states(&new_state, prev_state, desired_addresses);

        assert_eq!(merged_state, new_state);
    }

    /// Any changes equal to the desired state should be ignored. Other changes should be recorded.
    #[test]
    fn test_backup_ignore_desired_state() {
        let prev_state = HashMap::from([
            ("a".to_owned(), None),
            (
                "b".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["1.2.3.4".to_owned()],
                    "iface_b".to_owned(),
                )),
            ),
            (
                "c".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["10.64.0.1".to_owned()],
                    "iface_c".to_owned(),
                )),
            ),
            (
                "d".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["1.3.3.7".to_owned()],
                    "iface_d".to_owned(),
                )),
            ),
        ]);
        let new_state = HashMap::from([
            // This change should be ignored
            (
                "a".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["10.64.0.1".to_owned()],
                    "iface_a".to_owned(),
                )),
            ),
            // This change should be ignored
            (
                "b".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["10.64.0.1".to_owned()],
                    "iface_b".to_owned(),
                )),
            ),
            // This change should be ignored
            (
                "c".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["4.3.2.1".to_owned()],
                    "iface_c".to_owned(),
                )),
            ),
            // This change should NOT be ignored
            (
                "d".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["4.3.2.1".to_owned()],
                    "iface_d".to_owned(),
                )),
            ),
        ]);
        let expect_state = HashMap::from([
            ("a".to_owned(), None),
            (
                "b".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["1.2.3.4".to_owned()],
                    "iface_b".to_owned(),
                )),
            ),
            (
                "c".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["4.3.2.1".to_owned()],
                    "iface_c".to_owned(),
                )),
            ),
            (
                "d".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["4.3.2.1".to_owned()],
                    "iface_d".to_owned(),
                )),
            ),
        ]);

        let desired_addresses: BTreeSet<String> = ["10.64.0.1".to_owned()].into();

        let merged_state = State::merge_states(&new_state, prev_state, desired_addresses);

        assert_eq!(merged_state, expect_state);
    }

    /// Services not specified in the new state should be removed from the backed up state
    #[test]
    fn test_backup_remove_dns_config() {
        let prev_state = HashMap::from([
            (
                "a".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["10.64.0.1".to_owned()],
                    "iface_a".to_owned(),
                )),
            ),
            (
                "b".to_owned(),
                Some(DnsSettings::from_server_addresses(
                    &["1.2.3.4".to_owned()],
                    "iface_b".to_owned(),
                )),
            ),
            ("c".to_owned(), None),
        ]);
        let new_state = HashMap::from([("c".to_owned(), None)]);
        let expected_state = new_state.clone();

        let desired_addresses: BTreeSet<String> = ["10.64.0.1".to_owned()].into();

        let merged_state = State::merge_states(&new_state, prev_state, desired_addresses);

        assert_eq!(merged_state, expected_state);
    }

    /// If DHCP provides an IP identical to our desired state, the tracked state will not reflect
    /// this. This is a known limitation.
    // TODO: This should actually succeed. If we happen to switch to a network whose IP equals
    //       the "desired IP", we should still back up the result.
    #[test]
    #[should_panic]
    fn test_backup_change_equals_desired_state() {
        let prev_state = HashMap::from([(
            "a".to_owned(),
            Some(DnsSettings::from_server_addresses(
                &["192.168.100.1".to_owned()],
                "iface_a".to_owned(),
            )),
        )]);
        let new_state = HashMap::from([(
            "a".to_owned(),
            Some(DnsSettings::from_server_addresses(
                &["192.168.1.1".to_owned()],
                "iface_a".to_owned(),
            )),
        )]);
        let expect_state = HashMap::from([(
            "a".to_owned(),
            Some(DnsSettings::from_server_addresses(
                &["192.168.1.1".to_owned()],
                "iface_a".to_owned(),
            )),
        )]);

        let desired_addresses: BTreeSet<String> = ["192.168.1.1".to_owned()].into();

        let merged_state = State::merge_states(&new_state, prev_state, desired_addresses);

        assert_eq!(merged_state, expect_state);
    }
}
