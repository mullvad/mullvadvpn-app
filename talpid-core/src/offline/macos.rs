use crate::tunnel_state_machine::TunnelCommand;
use futures01::sync::mpsc::UnboundedSender;
use log::{debug, trace};
use std::{
    sync::{mpsc, Weak},
    thread,
};
use system_configuration::{
    core_foundation::{
        array::CFArray,
        runloop::{kCFRunLoopCommonModes, CFRunLoop},
        string::CFString,
    },
    dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder, SCDynamicStoreCallBackContext},
};


const PRIMARY_INTERFACE_KEY: &str = "State:/Network/Global/IPv4";


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize dynamic store")]
    DynamicStoreInitError,
}

pub struct MonitorHandle;

pub fn spawn_monitor(sender: Weak<UnboundedSender<TunnelCommand>>) -> Result<MonitorHandle, Error> {
    let (result_tx, result_rx) = mpsc::channel();
    thread::spawn(move || match create_dynamic_store(sender) {
        Ok(store) => {
            result_tx.send(Ok(())).unwrap();
            run_dynamic_store_runloop(store);
            log::error!("Core Foundation main loop exited! It should run forever");
        }
        Err(e) => result_tx.send(Err(e)).unwrap(),
    });
    result_rx.recv().unwrap().map(|_| MonitorHandle)
}

impl MonitorHandle {
    pub fn is_offline(&self) -> bool {
        let store = SCDynamicStoreBuilder::new("talpid-primary-interface").build();
        let is_offline = store.get(CFString::new(PRIMARY_INTERFACE_KEY)).is_none();
        is_offline
    }
}

fn create_dynamic_store(
    sender: Weak<UnboundedSender<TunnelCommand>>,
) -> Result<SCDynamicStore, Error> {
    let callback_context = SCDynamicStoreCallBackContext {
        callout: primary_interface_change_callback,
        info: sender,
    };

    let store = SCDynamicStoreBuilder::new("talpid-primary-interface")
        .callback_context(callback_context)
        .build();

    let watch_keys = CFArray::from_CFTypes(&[CFString::new(PRIMARY_INTERFACE_KEY)]);
    let watch_patterns: CFArray<CFString> = CFArray::from_CFTypes(&[]);

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

    trace!("Entering primary interface CFRunLoop");
    CFRunLoop::run_current();
}

fn primary_interface_change_callback(
    store: SCDynamicStore,
    _changed_keys: CFArray<CFString>,
    state: &mut Weak<UnboundedSender<TunnelCommand>>,
) {
    let is_offline = store.get(CFString::new(PRIMARY_INTERFACE_KEY)).is_none();
    debug!(
        "Computer went {}",
        if is_offline { "offline" } else { "online" }
    );
    if let Some(state) = state.upgrade() {
        let _ = state.unbounded_send(TunnelCommand::IsOffline(is_offline));
    }
}
