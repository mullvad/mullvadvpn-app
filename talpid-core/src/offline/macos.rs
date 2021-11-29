use futures::channel::mpsc::UnboundedSender;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Weak,
    },
    thread,
};
use system_configuration::{
    core_foundation::{
        array::CFArray,
        base::{CFType, TCFType, ToVoid},
        boolean::CFBoolean,
        dictionary::CFDictionary,
        runloop::{kCFRunLoopCommonModes, CFRunLoop},
        string::CFString,
    },
    dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder, SCDynamicStoreCallBackContext},
    network_configuration::{self, SCNetworkInterface, SCNetworkInterfaceType},
    network_reachability::{
        ReachabilityFlags, SCNetworkReachability, SchedulingError, SetCallbackError,
    },
};

const PRIMARY_INTERFACE_KEY: &str = "State:/Network/Global/IPv4";

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize dynamic store")]
    DynamicStoreInitError,
    #[error(display = "Failed to schedule reachability callback")]
    ScheduleReachabilityCallbackError(#[error(source)] SchedulingError),
    #[error(display = "Failed to set reachability callback")]
    SetCallbackError(#[error(source)] SetCallbackError),
    #[error(display = "Panic during initialization")]
    InitializationError,
}

pub struct MonitorHandle {
    _notify_tx: Arc<UnboundedSender<bool>>,
}

impl MonitorHandle {
    /// Host is considered to be offline if the IPv4 internet is considered to be unreachable by the
    /// given reachability flags *or* there are no active physical interfaces.
    pub async fn is_offline(&self) -> bool {
        let reachability = SCNetworkReachability::from(ipv4_internet());
        let store = SCDynamicStoreBuilder::new("talpid-offline-check").build();
        reachability
            .reachability()
            .map(|flags| check_offline_state(&store, flags))
            .unwrap_or(false)
    }
}

pub async fn spawn_monitor(notify_tx: UnboundedSender<bool>) -> Result<MonitorHandle, Error> {
    let (result_tx, result_rx) = mpsc::channel();
    let notify_tx = Arc::new(notify_tx);
    let sender = Arc::downgrade(&notify_tx);
    thread::spawn(move || {
        let mut reachability_ref = SCNetworkReachability::from(ipv4_internet());
        let store = SCDynamicStoreBuilder::new("talpid-offline-watcher").build();

        let is_currently_offline = match reachability_ref.reachability() {
            Ok(flags) => check_offline_state(&store, flags),
            Err(_) => {
                log::error!("Failed to obtain current connectivity, assuming machine is online");
                false
            }
        };

        let context = OfflineStateContext {
            sender,
            is_offline: Arc::new(AtomicBool::new(is_currently_offline)),
        };

        let result = || -> Result<SCDynamicStore, Error> {
            let dynamic_store = create_dynamic_store(context.clone())?;
            CFRunLoop::get_current().add_source(&dynamic_store.create_run_loop_source(), unsafe {
                kCFRunLoopCommonModes
            });

            reachability_ref.set_callback(move |flags| {
                let store = SCDynamicStoreBuilder::new("talpid-offline-watcher").build();
                context.new_state(check_offline_state(&store, flags));
            })?;

            reachability_ref.schedule_with_runloop(&CFRunLoop::get_current(), unsafe {
                kCFRunLoopCommonModes
            })?;

            Ok(dynamic_store)
        };

        match result() {
            Ok(_dynamic_store) => {
                let _ = result_tx.send(Ok(()));
                CFRunLoop::run_current()
            }
            Err(err) => {
                let _ = result_tx.send(Err(err));
            }
        }
    });

    let _ = result_rx.recv().map_err(|_| Error::InitializationError)??;
    Ok(MonitorHandle {
        _notify_tx: notify_tx,
    })
}

fn ipv4_internet() -> SocketAddr {
    SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
}

fn check_offline_state(store: &SCDynamicStore, flags: ReachabilityFlags) -> bool {
    let is_offline =
        !flags.contains(ReachabilityFlags::REACHABLE) || !exists_active_physical_iface(store);
    is_offline
}

fn exists_active_physical_iface(store: &SCDynamicStore) -> bool {
    network_configuration::get_interfaces().iter().any(|iface| {
        let is_physical = iface_is_physical(&*iface);
        let is_active = iface_is_active(&*iface, store);
        let is_valid = is_physical && is_active;
        if is_valid {
            log::trace!(
                "Considering interface {:?} {:?} to be active and physical",
                iface.bsd_name(),
                iface.display_name()
            );
        }
        is_valid
    })
}

fn iface_is_active(iface: &SCNetworkInterface, store: &SCDynamicStore) -> bool {
    || -> Option<bool> {
        let path = format!("State:/Network/Interface/{}/Link", iface.bsd_name()?);
        let link_properties = store
            .get(CFString::from(path.as_ref()))?
            .downcast::<CFDictionary>()?;

        let active_ptr = link_properties.find(CFString::from("Active").to_void())?;
        if active_ptr.is_null() {
            return None;
        }

        unsafe { CFType::wrap_under_get_rule(*active_ptr) }
            .downcast::<CFBoolean>()
            .map(Into::into)
    }()
    .unwrap_or(false)
}

fn iface_is_physical(iface: &SCNetworkInterface) -> bool {
    use SCNetworkInterfaceType::*;
    match iface.interface_type() {
        Some(iface_type) => match iface_type {
            Bluetooth | Modem | Serial | IrDA | Ethernet | FireWire | WWAN | IEEE80211 => true,
            _ => false,
        },
        // if interface type is unknown, have to assume it provides internet
        None => true,
    }
}

#[derive(Clone)]
struct OfflineStateContext {
    sender: Weak<UnboundedSender<bool>>,
    is_offline: Arc<AtomicBool>,
}

impl OfflineStateContext {
    fn no_primary_interface(&self) {
        self.new_state(true);
    }

    fn new_state(&self, is_offline: bool) {
        if self.is_offline.swap(is_offline, Ordering::SeqCst) != is_offline {
            if let Some(sender) = self.sender.upgrade() {
                let _ = sender.unbounded_send(is_offline);
            }
        }
    }
}

fn create_dynamic_store(context: OfflineStateContext) -> Result<SCDynamicStore, Error> {
    let callback_context = SCDynamicStoreCallBackContext {
        callout: primary_interface_change_callback,
        info: context,
    };

    let store = SCDynamicStoreBuilder::new("talpid-primary-interface")
        .callback_context(callback_context)
        .build();

    let watch_keys = CFArray::from_CFTypes(&[CFString::new(PRIMARY_INTERFACE_KEY)]);
    let watch_patterns: CFArray<CFString> = CFArray::from_CFTypes(&[]);

    if store.set_notification_keys(&watch_keys, &watch_patterns) {
        log::trace!("Registered for dynamic store notifications");
        Ok(store)
    } else {
        Err(Error::DynamicStoreInitError)
    }
}

fn primary_interface_change_callback(
    store: SCDynamicStore,
    _changed_keys: CFArray<CFString>,
    state: &mut OfflineStateContext,
) {
    let is_offline = store.get(CFString::new(PRIMARY_INTERFACE_KEY)).is_none();
    if is_offline {
        log::debug!("No primary interface, considering host to be offline");
        state.no_primary_interface();
    }
}
