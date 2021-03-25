use crate::linux::{iface_index, IfaceIndexLookupError};
use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
use talpid_dbus::systemd_resolved::{DnsState, SystemdResolved as DbusInterface};

pub(crate) use talpid_dbus::systemd_resolved::Error as SystemdDbusError;

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
    state: Option<SetConfigState>,
}

struct SetConfigState {
    dns_config: Arc<DnsState>,
    watcher_thread: thread::JoinHandle<()>,
    watcher_should_shutdown: Arc<AtomicBool>,
}


impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?;

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            state: None,
        };

        Ok(systemd_resolved)
    }

    pub fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let iface_index = iface_index(interface_name)?;
        let dns_state = self.dbus_interface.set_dns(iface_index, servers)?;
        let dns_config = Arc::new(dns_state);


        let (watcher_thread, watcher_should_shutdown) =
            self.spawn_watcher_thread(dns_config.clone());
        self.state = Some(SetConfigState {
            dns_config,
            watcher_thread,
            watcher_should_shutdown,
        });


        Ok(())
    }

    fn spawn_watcher_thread(
        &mut self,
        dns_state: Arc<DnsState>,
    ) -> (thread::JoinHandle<()>, Arc<AtomicBool>) {
        let dbus_interface = self.dbus_interface.clone();
        let should_shutdown = Arc::new(AtomicBool::new(false));
        let watch_shutdown = should_shutdown.clone();
        let callback_shutdown = should_shutdown.clone();
        let watcher_thread = std::thread::spawn(move || {
            let result = dbus_interface.clone().watch_dns_changes(
                move |new_servers| {
                    if callback_shutdown.clone().load(Ordering::Acquire) {
                        return;
                    }
                    let mut current_servers: Vec<IpAddr> = new_servers
                            .into_iter()
                            .filter(|server| server.iface_index == dns_state.interface_index as i32)
                            .map(|server| server.address)
                            .collect();
                    current_servers.sort();
                    if current_servers != *dns_state.set_servers {
                        log::debug!("DNS config for tunnel interface changed, currently applied servers - {:?}", current_servers);
                        if let Err(err) = dbus_interface.set_dns(dns_state.interface_index, &dns_state.set_servers) {
                            log::error!("Failed to re-apply DNS config - {}", err);
                        }
                    }
                },
                move || !watch_shutdown.load(Ordering::Acquire),
            );
            if let Err(err) = result {
                log::error!("Failed to watch DNS config updates: {}", err);
            }
        });
        (watcher_thread, should_shutdown)
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(SetConfigState {
            dns_config,
            watcher_thread,
            watcher_should_shutdown,
        }) = self.state.take()
        {
            watcher_should_shutdown.store(true, Ordering::Release);
            if let Err(err) = self.dbus_interface.revert_link(&dns_config) {
                log::error!("Failed to revert interface config: {}", err);
            }

            if watcher_thread.join().is_err() {
                log::error!("DNS watcher thread panicked!");
            }
        }

        Ok(())
    }
}
