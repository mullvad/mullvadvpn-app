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
use talpid_dbus::systemd_resolved::{AsyncHandle, DnsState, SystemdResolved as DbusInterface};
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

use super::routing::{DnsConfig, DnsRouteMonitor};

pub struct SystemdResolved {
    pub dbus_interface: AsyncHandle,
    current_config: Arc<Mutex<Vec<DnsConfig>>>,
    initial_states: Arc<Mutex<Vec<DnsState>>>,
    tunnel_index: u32,
    route_monitor: Option<(DnsRouteMonitor, tokio::task::JoinHandle<()>)>,
    watcher: Option<(thread::JoinHandle<()>, Arc<AtomicBool>)>,
}


impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            current_config: Arc::new(Mutex::new(vec![])),
            initial_states: Arc::new(Mutex::new(vec![])),
            tunnel_index: 0,
            route_monitor: None,
            watcher: None,
        };

        Ok(systemd_resolved)
    }

    pub async fn set_dns(&mut self, interface_name: &str, servers: &[IpAddr]) -> Result<()> {
        let (update_tx, mut update_rx) = mpsc::unbounded();
        let (monitor, initial_config) = super::routing::spawn_monitor(servers.to_vec(), update_tx)
            .await
            .map_err(Error::SpawnInterfaceMonitor)?;

        let tunnel_index = iface_index(interface_name)?;
        self.tunnel_index = tunnel_index;
        let mut last_result = Ok(());

        for iface_config in &initial_config {
            let initial_state = match self.dbus_interface.get_dns(iface_config.interface).await {
                Ok(state) => state,
                Err(error) => {
                    last_result = Err(Error::SystemdResolvedError(error));
                    break;
                }
            };
            if let Err(error) = self
                .dbus_interface
                .set_dns(iface_config.interface, iface_config.resolvers.clone())
                .await
            {
                last_result = Err(Error::SystemdResolvedError(error));
                break;
            }
            self.initial_states.lock().unwrap().push(initial_state);
        }

        if last_result.is_ok() {
            if initial_config.len() == 1 && initial_config[0].interface == tunnel_index {
                if let Err(error) = self
                    .dbus_interface
                    .set_domains(tunnel_index, &[(".", true)])
                    .await
                {
                    last_result = Err(Error::SystemdResolvedError(error));
                }
            } else {
                if let Err(error) = self.dbus_interface.set_domains(tunnel_index, &[]).await {
                    last_result = Err(Error::SystemdResolvedError(error));
                }
            }
        }

        if let Err(error) = last_result {
            let _ = self.reset();
            return Err(error);
        }

        {
            *self.current_config.lock().unwrap() = initial_config;
        }

        let ignore_config_changes = Arc::new(AtomicBool::new(false));

        self.watcher = Some(self.spawn_watcher_thread(
            tunnel_index,
            self.current_config.clone(),
            ignore_config_changes.clone(),
        ));

        let dbus_interface = self.dbus_interface.clone();
        let initial_states = self.initial_states.clone();
        let current_config = self.current_config.clone();
        let join_handle = tokio::spawn(async move {
            while let Some(mut new_config) = update_rx.next().await {
                let mut new_initial_states = { initial_states.lock().unwrap().clone() };
                new_initial_states.sort_by(|a, b| a.interface_index.cmp(&b.interface_index));
                new_config.sort_by(|a, b| a.interface.cmp(&b.interface));

                let disable_watcher = ignore_config_changes.clone();
                disable_watcher.store(true, Ordering::Release);

                // Revert interfaces no longer in use
                let mut state_index = new_initial_states.len();
                while state_index > 0 {
                    let state = &new_initial_states[state_index - 1];
                    match new_config.binary_search_by(|a| a.interface.cmp(&state.interface_index)) {
                        Ok(_) => state_index -= 1,
                        Err(_index) => {
                            log::debug!(
                                "Reverting DNS config on interface {}",
                                state.interface_index
                            );
                            if let Err(err) = dbus_interface.set_dns_state(state.clone()).await {
                                log::error!("Failed to revert interface config: {}", err);
                            }
                            new_initial_states.remove(state_index - 1);
                        }
                    }
                }

                for iface_config in &new_config {
                    if tunnel_index == iface_config.interface {
                        // All public addresses (plus the gateway) will be assigned
                        // to the tunnel: we can assume nothing has changed.
                        continue;
                    }

                    // Store new interfaces
                    if let Err(index) = new_initial_states
                        .binary_search_by(|a| a.interface_index.cmp(&iface_config.interface))
                    {
                        let initial_state =
                            match dbus_interface.get_dns(iface_config.interface).await {
                                Ok(state) => state,
                                Err(error) => {
                                    log::error!(
                                        "Failed to get resolvers: {}\n{}",
                                        iface_config,
                                        error.display_chain()
                                    );
                                    continue;
                                }
                            };
                        new_initial_states.insert(index, initial_state);
                    }

                    if let Err(error) = dbus_interface
                        .set_dns(iface_config.interface, iface_config.resolvers.clone())
                        .await
                    {
                        log::error!(
                            "Failed to set resolvers: {}\n{}",
                            iface_config,
                            error.display_chain()
                        );
                    }
                }

                let tunnel_domains =
                    if new_config.len() == 1 && new_config[0].interface == tunnel_index {
                        &[(".", true)][..]
                    } else {
                        &[][..]
                    };
                if let Err(error) = dbus_interface
                    .set_domains(tunnel_index, tunnel_domains)
                    .await
                {
                    log::error!(
                        "Failed to set DNS domains: {}\n{}",
                        new_config[0],
                        error.display_chain()
                    );
                }

                {
                    *current_config.lock().unwrap() = new_config.clone();
                    *initial_states.lock().unwrap() = new_initial_states;
                }

                disable_watcher.store(false, Ordering::Release);
            }
        });
        self.route_monitor = Some((monitor, join_handle));

        Ok(())
    }

    fn spawn_watcher_thread(
        &mut self,
        tunnel_index: u32,
        current_config: Arc<Mutex<Vec<DnsConfig>>>,
        disable_watcher: Arc<AtomicBool>,
    ) -> (thread::JoinHandle<()>, Arc<AtomicBool>) {
        let dbus_interface = self.dbus_interface.handle().clone();
        let should_shutdown = Arc::new(AtomicBool::new(false));
        let watch_shutdown = should_shutdown.clone();
        let callback_shutdown = should_shutdown.clone();
        let watcher_thread = std::thread::spawn(move || {
            let result = dbus_interface.clone().watch_dns_changes(
                move |new_servers| {
                    if callback_shutdown.clone().load(Ordering::Acquire) {
                        return;
                    }
                    if disable_watcher.clone().load(Ordering::Acquire) {
                        return;
                    }
                    let configs = current_config.lock().unwrap();
                    let mut anything_changed = false;
                    for config in &*configs {
                        let current_servers: Vec<IpAddr> = new_servers
                            .iter()
                            .filter(|server| server.iface_index == config.interface as i32)
                            .map(|server| server.address)
                            .collect();
                        if current_servers != config.resolvers {
                            log::debug!("DNS config for interface {} changed, currently applied servers - {:?}", config.interface, current_servers);
                            if let Err(err) = dbus_interface.set_dns(config.interface, config.resolvers.clone())
                            {
                                log::error!("Failed to re-apply DNS config - {}", err);
                            }
                            anything_changed = true;
                        }
                    }
                    if anything_changed {
                        let result = if configs.len() == 1 && configs[0].interface == tunnel_index {
                            dbus_interface.set_domains(tunnel_index, &[(".", true)])
                        } else {
                            dbus_interface.set_domains(tunnel_index, &[])
                        };
                        if let Err(err) = result {
                            log::error!("Failed to re-apply DNS domains - {}", err);
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

    pub async fn reset(&mut self) -> Result<()> {
        if let Some((watcher_thread, watcher_should_shutdown)) = self.watcher.take() {
            watcher_should_shutdown.store(true, Ordering::Release);
            if watcher_thread.join().is_err() {
                log::error!("DNS watcher thread panicked!");
            }
        }

        if let Some((monitor, join_handle)) = self.route_monitor.take() {
            std::mem::drop(monitor);
            let _ = join_handle.await;
        }

        for state in self.initial_states.lock().unwrap().drain(..) {
            let result = if state.interface_index == self.tunnel_index {
                self.dbus_interface.revert_link(state.clone()).await
            } else {
                self.dbus_interface.set_dns_state(state).await
            };
            if let Err(err) = result {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to revert interface config")
                );
            }
        }

        self.current_config.lock().unwrap().clear();

        Ok(())
    }
}
