extern crate core_foundation;
extern crate system_configuration;

use self::core_foundation::array::CFArray;
use self::core_foundation::base::{CFType, TCFType};
use self::core_foundation::dictionary::CFDictionary;
use self::core_foundation::propertylist::CFPropertyList;
use self::core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use self::core_foundation::string::CFString;

use self::system_configuration::dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder,
                                                SCDynamicStoreCallBackContext};

use std::collections::HashMap;
use std::mem;
use std::net::IpAddr;
use std::sync::mpsc;
use std::thread;

use dns::{DnsConfig, DnsConfigInterface, DnsConfigManager, DnsConfigMonitor, UpdateSender};

error_chain! {
    errors {
        SettingDnsFailed { description("Error while setting DNS servers") }
        DynamicStoreInitError { description("Failed to initialize dynamic store") }
    }
}

const STATE_PATH_PATTERN: &str = "State:/Network/Service/.*/DNS";
const SETUP_PATH_PATTERN: &str = "Setup:/Network/Service/.*/DNS";

type ServicePath = String;
type DnsServer = String;

pub type MacOsDnsManager = DnsConfigManager<MacOsDnsInterface, MacOsDnsMonitor>;

#[derive(Clone)]
pub struct MacOsDnsConfig {
    service_configs: HashMap<ServicePath, Option<Vec<DnsServer>>>,
}

impl From<HashMap<ServicePath, Option<Vec<DnsServer>>>> for MacOsDnsConfig {
    fn from(service_configs: HashMap<ServicePath, Option<Vec<DnsServer>>>) -> Self {
        MacOsDnsConfig { service_configs }
    }
}

impl DnsConfig for MacOsDnsConfig {
    fn uses_nameservers(&self, nameservers: &Vec<IpAddr>) -> bool {
        let nameserver_strings = Some(nameservers.iter().map(IpAddr::to_string).collect());

        self.service_configs
            .values()
            .all(|service_config| *service_config == nameserver_strings)
    }

    fn set_nameservers(&mut self, nameservers: &Vec<IpAddr>) {
        let new_service_config = Some(nameservers.iter().map(IpAddr::to_string).collect());

        for service_config in self.service_configs.values_mut() {
            mem::replace(service_config, new_service_config.clone());
        }
    }

    fn merge_with(&mut self, other: Self) {
        self.service_configs.extend(other.service_configs);
    }

    fn merge_ignoring_nameservers(&mut self, _: Self) {}
}

pub struct MacOsDnsInterface {
    store: SCDynamicStore,
}

impl DnsConfigInterface for MacOsDnsInterface {
    type Config = MacOsDnsConfig;
    type Update = Vec<String>;
    type Error = Error;

    fn open() -> Result<Self> {
        Ok(MacOsDnsInterface {
            store: SCDynamicStoreBuilder::new("mullvad-dns").build(),
        })
    }

    fn read_config(&mut self) -> Result<Self::Config> {
        Ok(read_all_dns(&self.store).into())
    }

    fn write_config(&mut self, config: Self::Config) -> Result<()> {
        for (service, service_config) in config.service_configs {
            if let Some(service_config) = service_config {
                set_dns(&self.store, CFString::new(&service), &service_config)?;
            } else {
                if !self.store.remove(CFString::new(&service)) {
                    bail!(ErrorKind::SettingDnsFailed);
                }
            }
        }

        Ok(())
    }

    fn read_update(&mut self, changed_keys: Self::Update) -> Result<Self::Config> {
        let mut service_configs = HashMap::new();

        for key in changed_keys {
            let value = read_dns(&self.store, CFString::new(&key));

            service_configs.insert(key, value);
        }

        Ok(MacOsDnsConfig { service_configs })
    }
}

pub struct MacOsDnsMonitor;

impl DnsConfigMonitor<Vec<String>> for MacOsDnsMonitor {
    type Error = Error;

    fn spawn(event_tx: UpdateSender<Vec<String>>) -> Result<Self> {
        let (result_tx, result_rx) = mpsc::channel();
        thread::spawn(move || match create_dynamic_store(event_tx) {
            Ok(store) => {
                result_tx.send(Ok(())).unwrap();
                run_dynamic_store_runloop(store);
                // TODO(linus): This is critical. Improve later by sending error signal to Daemon
                error!("Core Foundation main loop exited! It should run forever");
            }
            Err(e) => result_tx.send(Err(e)).unwrap(),
        });
        result_rx.recv().unwrap()?;
        Ok(MacOsDnsMonitor)
    }
}

/// Creates a `SCDynamicStore` that watches all network interfaces for changes to the DNS settings.
fn create_dynamic_store(listener: UpdateSender<Vec<String>>) -> Result<SCDynamicStore> {
    let callback_context = SCDynamicStoreCallBackContext {
        callout: dns_change_callback,
        info: listener,
    };

    let store = SCDynamicStoreBuilder::new("mullvad-dns-monitor")
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
        bail!(ErrorKind::DynamicStoreInitError)
    }
}

fn run_dynamic_store_runloop(store: SCDynamicStore) {
    let run_loop_source = store.create_run_loop_source();
    CFRunLoop::get_current().add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });

    trace!("Entering CFRunLoop");
    CFRunLoop::run_current();
}

/// This function is called by the Core Foundation event loop when there is a change to one or more
/// watched dynamic store values. In our case we watch all DNS settings.
fn dns_change_callback(
    _store: SCDynamicStore,
    changed_keys: CFArray<CFString>,
    listener: &mut UpdateSender<Vec<String>>,
) {
    let mut change = Vec::new();

    for key in &changed_keys {
        let state_path = key.to_string();
        let converted_setup_path = state_to_setup_path(&state_path);

        change.push(state_path);

        if let Some(setup_path) = converted_setup_path {
            change.push(setup_path);
        }
    }

    let _ = listener.send(change);
}

/// Set the dynamic store entry at `path` to a dictionary with the given servers under the
/// "ServerAddresses" key.
fn set_dns(store: &SCDynamicStore, path: CFString, servers: &[DnsServer]) -> Result<()> {
    debug!("Setting DNS to [{}] for {}", servers.join(", "), path);
    let server_addresses_key = CFString::from_static_string("ServerAddresses");

    let cf_string_servers: Vec<CFString> = servers.iter().map(|s| CFString::new(s)).collect();
    let server_addresses_value = CFArray::from_CFTypes(&cf_string_servers);

    let dns_dictionary =
        CFDictionary::from_CFType_pairs(&[(server_addresses_key, server_addresses_value)]);

    if store.set(path, dns_dictionary) {
        Ok(())
    } else {
        bail!(ErrorKind::SettingDnsFailed)
    }
}

/// Read all existing DNS settings and return them.
fn read_all_dns(store: &SCDynamicStore) -> HashMap<ServicePath, Option<Vec<DnsServer>>> {
    let mut backup = HashMap::new();
    // Backup all "state" DNS, and all corresponding "setup" DNS even if they don't exist
    if let Some(paths) = store.get_keys(STATE_PATH_PATTERN) {
        for state_path in paths.iter() {
            let state_path_str = state_path.to_string();
            let setup_path_str = state_to_setup_path(&state_path_str).unwrap();
            let setup_path = CFString::new(&setup_path_str);
            backup.insert(state_path_str, read_dns(store, state_path.clone()));
            backup.insert(setup_path_str, read_dns(store, setup_path));
        }
    }
    // Backup all "setup" DNS not already covered
    if let Some(paths) = store.get_keys(SETUP_PATH_PATTERN) {
        for setup_path in paths.iter() {
            let setup_path_str = setup_path.to_string();
            if !backup.contains_key(&setup_path_str) {
                backup.insert(setup_path_str, read_dns(store, setup_path.clone()));
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

/// Get DNS settings for a given dynamic store path. Returns `None` If the path does not exist
/// or does not contain the expected format.
fn read_dns(store: &SCDynamicStore, path: CFString) -> Option<Vec<DnsServer>> {
    store
        .get(path.clone())
        .and_then(CFPropertyList::downcast_into::<CFDictionary>)
        .and_then(|dictionary| {
            dictionary
                .find2(&CFString::from_static_string("ServerAddresses"))
                .map(|array_ptr| unsafe { CFType::wrap_under_get_rule(array_ptr) })
        })
        .and_then(|addresses| {
            if let Some(array) = addresses.downcast::<CFArray<CFType>>() {
                parse_cf_array_to_strings(array)
            } else {
                error!("DNS ServerAddresess is not an array: {:?}", addresses);
                None
            }
        })
}

/// Parses a CFArray into a Rust vector of Rust strings, if the array contains CFString instances
/// only, otherwise `None` is returned.
fn parse_cf_array_to_strings(array: CFArray<CFType>) -> Option<Vec<String>> {
    let mut strings = Vec::new();
    for item in array.iter() {
        if let Some(string) = item.downcast::<CFString>() {
            strings.push(string.to_string());
        } else {
            error!("DNS server entry is not a string: {:?}", item);
            return None;
        };
    }
    Some(strings)
}
