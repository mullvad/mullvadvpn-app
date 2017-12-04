extern crate core_foundation;
extern crate system_configuration;

use self::core_foundation::array::{CFArray, CFArrayRef};
use self::core_foundation::base::{CFType, TCFType};
use self::core_foundation::dictionary::CFDictionary;
use self::core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use self::core_foundation::string::{CFString, CFStringRef};

use self::system_configuration::dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder,
                                                SCDynamicStoreCallBackContext};

use error_chain::ChainedError;

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

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

struct State {
    desired_dns: Vec<DnsServer>,
    backup: HashMap<ServicePath, Option<Vec<DnsServer>>>,
}

pub struct DnsMonitor {
    store: SCDynamicStore,

    /// The current DNS injection state. If this is `None` it means we are not injecting any DNS.
    /// When it's `Some(state)` we are actively making sure `state.desired_dns` is configured
    /// on all network interfaces.
    state: Arc<Mutex<Option<State>>>,
}

impl DnsMonitor {
    /// Creates and returns a new `DnsMonitor`. This spawns a background thread that will monitor
    /// DNS settings for all network interfaces. If any changes occur it will instantly reset
    /// the DNS settings for that interface back to the last server list set to this instance
    /// with `set_dns`.
    pub fn new() -> Result<Self> {
        let state = Arc::new(Mutex::new(None));
        Self::spawn(state.clone())?;
        Ok(DnsMonitor {
            store: SCDynamicStoreBuilder::new("mullvad-dns").build(),
            state,
        })
    }

    /// Spawns the background thread running the CoreFoundation main loop and monitors the system
    /// for DNS changes.
    fn spawn(state: Arc<Mutex<Option<State>>>) -> Result<()> {
        let (result_tx, result_rx) = mpsc::channel();
        thread::spawn(move || match create_dynamic_store(state) {
            Ok(store) => {
                result_tx.send(Ok(())).unwrap();
                run_dynamic_store_runloop(store);
                // TODO(linus): This is critical. Improve later by sending error signal to Daemon
                error!("Core Foundation main loop exited! It should run forever");
            }
            Err(e) => result_tx.send(Err(e)).unwrap(),
        });
        result_rx.recv().unwrap()
    }

    pub fn set_dns(&self, servers: Vec<DnsServer>) -> Result<()> {
        let mut state_lock = self.state.lock().unwrap();
        *state_lock = Some(match state_lock.take() {
            None => {
                debug!("Setting DNS to [{}]", servers.join(", "));
                let backup = read_all_dns(&self.store);
                for service_path in backup.keys() {
                    set_dns(&self.store, CFString::new(service_path), &servers)?;
                }
                State {
                    desired_dns: servers,
                    backup,
                }
            }
            Some(state) => if servers != state.desired_dns {
                debug!("Changing DNS to [{}]", servers.join(", "));
                for service_path in state.backup.keys() {
                    set_dns(&self.store, CFString::new(service_path), &servers)?;
                }
                State {
                    desired_dns: servers,
                    backup: state.backup,
                }
            } else {
                debug!("No change, new DNS same as the one already set");
                state
            },
        });
        Ok(())
    }

    /// Reset all DNS settings to the latest backed up values.
    pub fn reset(&self) -> Result<()> {
        let mut state_lock = self.state.lock().unwrap();
        if let Some(state) = state_lock.take() {
            for (service_path, servers) in state.backup {
                if let Some(servers) = servers {
                    set_dns(&self.store, CFString::new(&service_path), &servers)?;
                } else {
                    debug!("Removing DNS for {}", service_path);
                    if !self.store.remove(CFString::new(&service_path)) {
                        bail!(ErrorKind::SettingDnsFailed);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Creates a `SCDynamicStore` that watches all network interfaces for changes to the DNS settings.
fn create_dynamic_store(state: Arc<Mutex<Option<State>>>) -> Result<SCDynamicStore> {
    let callback_context = SCDynamicStoreCallBackContext {
        callout: dns_change_callback,
        info: state,
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
    store: SCDynamicStore,
    changed_keys: CFArray<CFString>,
    state: &mut Arc<Mutex<Option<State>>>,
) {
    if let Err(e) = dns_change_callback_internal(store, changed_keys, state) {
        error!("{}", e.display_chain());
    }
}

fn dns_change_callback_internal(
    store: SCDynamicStore,
    changed_keys: CFArray<CFString>,
    state: &mut Arc<Mutex<Option<State>>>,
) -> Result<()> {
    let mut state_lock = state.lock().unwrap();
    match *state_lock {
        None => {
            trace!("Not injecting DNS at this time");
        }
        Some(ref mut state) => for path_ptr in changed_keys.as_untyped().iter() {
            let path = unsafe { CFString::wrap_under_get_rule(path_ptr as CFStringRef) };
            let should_set_dns = match read_dns(&store, path.clone()) {
                None => {
                    debug!("Detected DNS removed for {}", path);
                    state.backup.insert(path.to_string(), None);
                    true
                }
                Some(servers) => if servers != state.desired_dns {
                    debug!(
                        "Detected DNS changed to [{}] for {}",
                        servers.join(", "),
                        path
                    );
                    state.backup.insert(path.to_string(), Some(servers));
                    true
                } else {
                    false
                },
            };
            if should_set_dns {
                set_dns(&store, path.clone(), &state.desired_dns)
                    .chain_err(|| format!("Failed changing DNS for {}", path))?;
                // If we changed a state DNS, also set the corresponding setup DNS.
                if let Some(setup_path_str) = state_to_setup_path(&path.to_string()) {
                    let setup_path = CFString::new(&setup_path_str);
                    if !state.backup.contains_key(&setup_path_str) {
                        state
                            .backup
                            .insert(setup_path_str, read_dns(&store, setup_path.clone()));
                    }
                    set_dns(&store, setup_path.clone(), &state.desired_dns)
                        .chain_err(|| format!("Failed changing DNS for {}", setup_path))?;
                }
            }
        },
    }
    Ok(())
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

    if store.set(path, &dns_dictionary) {
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
        for path_ptr in paths.as_untyped().iter() {
            let state_path =
                unsafe { CFString::wrap_under_get_rule(path_ptr as CFStringRef) };
            let state_path_str = state_path.to_string();
            let setup_path_str = state_to_setup_path(&state_path_str).unwrap();
            let setup_path = CFString::new(&setup_path_str);
            backup.insert(state_path_str, read_dns(store, state_path));
            backup.insert(setup_path_str, read_dns(store, setup_path));
        }
    }
    // Backup all "setup" DNS not already covered
    if let Some(paths) = store.get_keys(SETUP_PATH_PATTERN) {
        for path_ptr in paths.as_untyped().iter() {
            let setup_path =
                unsafe { CFString::wrap_under_get_rule(path_ptr as CFStringRef) };
            let setup_path_str = setup_path.to_string();
            if !backup.contains_key(&setup_path_str) {
                backup.insert(setup_path_str, read_dns(store, setup_path));
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
        .and_then(|property_list| property_list.downcast::<_, CFDictionary>())
        .and_then(|dictionary| {
            dictionary
                .find2(&CFString::from_static_string("ServerAddresses"))
                .map(|array_ptr| unsafe { CFType::wrap_under_get_rule(array_ptr) })
        })
        .and_then(|addresses: CFType| {
            if addresses.instance_of::<_, CFArray>() {
                let addresses_array = unsafe {
                    CFArray::wrap_under_get_rule(addresses.as_concrete_TypeRef() as CFArrayRef)
                };
                parse_cf_string_array(addresses_array)
            } else {
                error!("DNS settings is not an array: {:?}", addresses);
                None
            }
        })
}

/// Parses a CFArray into a Rust vector of Rust strings, if the array contains CFString instances,
/// otherwise `None` is returned.
fn parse_cf_string_array(array: CFArray) -> Option<Vec<String>> {
    let mut strings = Vec::new();
    for string_ptr in array.iter() {
        let cf_type = unsafe { CFType::wrap_under_get_rule(string_ptr) };
        if cf_type.instance_of::<_, CFString>() {
            let address =
                unsafe { CFString::wrap_under_get_rule(string_ptr as CFStringRef) };
            strings.push(address.to_string());
        } else {
            error!("DNS server entry is not a string: {:?}", cf_type);
            return None;
        };
    }
    Some(strings)
}
