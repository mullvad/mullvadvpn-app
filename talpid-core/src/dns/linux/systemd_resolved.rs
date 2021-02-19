use crate::linux::{iface_index, IfaceIndexLookupError};
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
    thread,
};
use talpid_dbus::systemd_resolved::{DnsState, SystemdResolved as DbusInterface};

pub(crate) use talpid_dbus::systemd_resolved::Error as SystemdDbusError;
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "systemd-resolved operation failed")]
    SystemdResolvedError(#[error(source)] SystemdDbusError),

    #[error(display = "Failed to resolve interface index with error {}", _0)]
    InterfaceNameError(#[error(source)] IfaceIndexLookupError),
}

pub struct SystemdResolved {
    pub dbus_interface: DbusInterface,
    state: Option<Arc<Mutex<Option<DnsState>>>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}


impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?;

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            state: None,
            watcher_thread: None,
        };

        Ok(systemd_resolved)
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let iface_index = iface_index(interface_name)?;
        let dns_state = self.dbus_interface.set_dns(iface_index, servers)?;
        let cloned_dns_state = self.set_dns_state(dns_state);
        let weak_dns_state = Arc::downgrade(&cloned_dns_state);
        let dns_state_should_continue = weak_dns_state.clone();

        let dbus_interface = self.dbus_interface.clone();
        let mut applied_servers: Vec<_> = servers.iter().cloned().collect();
        applied_servers.sort();
        let applied_servers = Arc::new(applied_servers);

        self.watcher_thread = Some(std::thread::spawn(move || {
            let result = dbus_interface.clone().watch_dns_changes(
                move |new_servers| {
                    (|| {
                        let dns_state_lock = weak_dns_state.upgrade()?;
                        let dns_state = dns_state_lock.lock().ok()?;
                        let dns_state_ref: &DnsState = &*dns_state.as_ref()?;

                        let mut current_servers: Vec<IpAddr> = new_servers
                                .into_iter()
                                .filter(|server| server.iface_index == iface_index as i32)
                                .map(|server| server.address)
                                .collect();
                        current_servers.sort();
                        if current_servers != *dns_state_ref.set_servers {
                            log::debug!("DNS config for tunnel interface changed, currently applied servers - {:?}", current_servers);
                            if let Err(err) = dbus_interface.set_dns(iface_index, &applied_servers) {
                                log::error!("Failed to re-apply DNS config - {}", err);
                            }
                        }
                        Some(())
                    })();
                },
                || dns_state_should_continue.upgrade().is_some(),
            );
            if let Err(err) = result {
                log::error!("Failed to watch DNS config updates: {}", err);
            }
        }));
        Ok(())
    }

    fn set_dns_state(&mut self, dns_state: DnsState) -> Arc<Mutex<Option<DnsState>>> {
        let new_state = Arc::new(Mutex::new(Some(dns_state)));
        self.state = Some(new_state.clone());
        new_state
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(state_lock) = self.state.take() {
            if let Some(dns_state) = state_lock.lock().expect("DNS state lock poisoned").take() {
                if let Err(err) = self.dbus_interface.revert_link(dns_state) {
                    log::error!("Failed to revert DNS config - {}", err.display_chain());
                }
            }
        } else {
            log::trace!("No DNS settings to reset");
        }
        if let Some(join_handle) = self.watcher_thread.take() {
            let _ = join_handle.join();
        }

        Ok(())
    }
}
