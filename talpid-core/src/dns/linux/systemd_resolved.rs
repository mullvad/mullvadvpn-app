use crate::linux::{iface_index, IfaceIndexLookupError};
use futures::{channel::mpsc, StreamExt};
use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};
use talpid_dbus::systemd_resolved::{DnsState, SystemdResolved as DbusInterface};
use talpid_types::ErrorExt;

pub(crate) use talpid_dbus::systemd_resolved::Error as SystemdDbusError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "systemd-resolved operation failed")]
    SystemdResolvedError(#[error(source)] SystemdDbusError),

    #[error(display = "Failed to resolve interface index with error {}", _0)]
    InterfaceNameError(#[error(source)] IfaceIndexLookupError),

    #[error(display = "Failed to spawn DNS interface monitor")]
    SpawnInterfaceMonitor(#[error(source)] super::routing::Error),
}

use super::routing::DnsRouteMonitor;

pub struct SystemdResolved {
    pub dbus_interface: DbusInterface,
    state: Option<SetConfigState>,
    local_configs: Arc<Mutex<Vec<DnsState>>>,
    _route_monitor: Option<DnsRouteMonitor>,
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
            local_configs: Arc::new(Mutex::new(vec![])),
            _route_monitor: None,
        };

        Ok(systemd_resolved)
    }

    pub async fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let (update_tx, mut update_rx) = mpsc::unbounded();
        let (monitor, initial_config) = super::routing::spawn_monitor(servers.to_vec(), update_tx)
            .await
            .map_err(Error::SpawnInterfaceMonitor)?;
        self._route_monitor = Some(monitor);

        let tunnel_index = iface_index(interface_name)?;
        let mut last_result = Ok(());

        for iface_config in &initial_config {
            let initial_state = match self.dbus_interface.get_dns(iface_config.interface) {
                Ok(state) => state,
                Err(error) => {
                    last_result = Err(Error::SystemdResolvedError(error));
                    break;
                }
            };
            let dns_state = match self.dbus_interface.set_dns(
                iface_config.interface,
                &iface_config.resolvers,
                tunnel_index == iface_config.interface,
            ) {
                Ok(state) => state,
                Err(error) => {
                    last_result = Err(Error::SystemdResolvedError(error));
                    break;
                }
            };
            if tunnel_index == iface_config.interface {
                let dns_config = Arc::new(dns_state);
                let (watcher_thread, watcher_should_shutdown) =
                    self.spawn_watcher_thread(dns_config.clone());
                self.state = Some(SetConfigState {
                    dns_config,
                    watcher_thread,
                    watcher_should_shutdown,
                });
            } else {
                self.local_configs.lock().unwrap().push(initial_state);
            }
        }

        if let Err(error) = last_result {
            let _ = self.reset();
            return Err(error);
        }

        let mut dbus_interface = self.dbus_interface.clone();
        let original_config = self.local_configs.clone();
        tokio::spawn(async move {
            while let Some(new_config) = update_rx.next().await {
                log::debug!("DNS config for non-tunnel interface changed");

                let mut config = original_config.lock().unwrap();
                for dns_state in &*config {
                    if let Err(err) = dbus_interface.revert_link(&dns_state) {
                        log::error!("Failed to revert interface config: {}", err);
                    }
                }

                config.sort_by(|a, b| a.interface_index.cmp(&b.interface_index));

                for iface_config in &new_config {
                    if tunnel_index == iface_config.interface {
                        // All public addresses (plus the gateway) will be assigned
                        // to the tunnel: we can assume nothing has changed.
                        continue;
                    }

                    if let Err(index) =
                        config.binary_search_by(|a| a.interface_index.cmp(&iface_config.interface))
                    {
                        let initial_state = match dbus_interface.get_dns(iface_config.interface) {
                            Ok(state) => state,
                            Err(error) => {
                                log::error!(
                                    "Failed to set resolvers: {}\n{}",
                                    iface_config,
                                    error.display_chain()
                                );
                                continue;
                            }
                        };
                        if index <= config.len() {
                            config.insert(index, initial_state);
                        }
                    }

                    if let Err(error) = dbus_interface.set_dns(
                        iface_config.interface,
                        &iface_config.resolvers,
                        false,
                    ) {
                        log::error!(
                            "Failed to set resolvers: {}\n{}",
                            iface_config,
                            error.display_chain()
                        );
                    }
                }
            }
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
                        if let Err(err) = dbus_interface.set_dns(dns_state.interface_index, &dns_state.set_servers, true) {
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

        for state in self.local_configs.lock().unwrap().drain(..) {
            if let Err(err) =
                self.dbus_interface
                    .set_dns(state.interface_index, &state.set_servers, false)
            {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to revert interface config")
                );
            }
        }

        Ok(())
    }
}
