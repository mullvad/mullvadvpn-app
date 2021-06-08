use crate::linux::{iface_index, IfaceIndexLookupError};
use futures::{channel::mpsc, StreamExt};
use std::{
    collections::BTreeMap,
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
    current_config: Arc<Mutex<BTreeMap<u32, DnsConfig>>>,
    initial_states: Arc<Mutex<BTreeMap<u32, DnsState>>>,
    tunnel_index: u32,
    route_monitor: Option<(DnsRouteMonitor, tokio::task::JoinHandle<()>)>,
    watcher: Option<(thread::JoinHandle<()>, Arc<AtomicBool>)>,
}


impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            current_config: Arc::new(Mutex::new(BTreeMap::new())),
            initial_states: Arc::new(Mutex::new(BTreeMap::new())),
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

        {
            let mut initial_states = self.initial_states.lock().unwrap();
            for (iface_index, iface_config) in &initial_config {
                let initial_state = match self.dbus_interface.get_dns(*iface_index).await {
                    Ok(state) => state,
                    Err(error) => {
                        last_result = Err(Error::SystemdResolvedError(error));
                        break;
                    }
                };
                if let Err(error) = self
                    .dbus_interface
                    .set_dns(*iface_index, iface_config.resolvers.clone())
                    .await
                {
                    last_result = Err(Error::SystemdResolvedError(error));
                    break;
                }
                initial_states.insert(*iface_index, initial_state);
            }
        }

        if last_result.is_ok() {
            if has_only_tunnel_config(&initial_config, tunnel_index) {
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

        let dbus_interface = DbusInterface::new_connection()?.async_handle();
        let initial_states = self.initial_states.clone();
        let current_config = self.current_config.clone();
        let join_handle = tokio::spawn(async move {
            while let Some(new_config) = update_rx.next().await {
                let mut new_initial_states = { initial_states.lock().unwrap().clone() };

                let disable_watcher = ignore_config_changes.clone();
                disable_watcher.store(true, Ordering::Release);

                // Revert interfaces no longer in use
                let keys = new_initial_states.keys().cloned().collect::<Vec<u32>>();
                for iface in keys {
                    if !new_config.contains_key(&iface) {
                        log::debug!("Reverting DNS config on interface {}", iface);
                        if let Err(err) = dbus_interface
                            .set_dns_state(new_initial_states[&iface].clone())
                            .await
                        {
                            log::error!("Failed to revert interface config: {}", err);
                        }
                        new_initial_states.remove(&iface);
                    }
                }

                for (iface, config) in &new_config {
                    if tunnel_index == *iface {
                        // All public addresses (plus the gateway) will be assigned
                        // to the tunnel: we can assume nothing has changed.
                        continue;
                    }

                    // Store new interfaces
                    if !new_initial_states.contains_key(iface) {
                        let initial_state = match dbus_interface.get_dns(*iface).await {
                            Ok(state) => state,
                            Err(error) => {
                                log::error!(
                                    "Failed to get resolvers: {}\n{}",
                                    config,
                                    error.display_chain()
                                );
                                continue;
                            }
                        };
                        new_initial_states.insert(*iface, initial_state);
                    }

                    if let Err(error) = dbus_interface
                        .set_dns(*iface, config.resolvers.clone())
                        .await
                    {
                        log::error!(
                            "Failed to set resolvers: {}\n{}",
                            config,
                            error.display_chain()
                        );
                    }
                }

                let tunnel_domains = if has_only_tunnel_config(&new_config, tunnel_index) {
                    &[(".", true)][..]
                } else {
                    &[][..]
                };
                if let Err(error) = dbus_interface
                    .set_domains(tunnel_index, tunnel_domains)
                    .await
                {
                    log::error!(
                        "Failed to set DNS domains on tunnel interface\n{}",
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
        current_config: Arc<Mutex<BTreeMap<u32, DnsConfig>>>,
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
                    for (iface, config) in &*configs {
                        let current_servers: Vec<IpAddr> = new_servers
                            .iter()
                            .filter(|server| server.iface_index == *iface as i32)
                            .map(|server| server.address)
                            .collect();
                        if current_servers != config.resolvers {
                            log::trace!("DNS config for interface {} changed, currently applied servers - {:?}", iface, current_servers);
                            if let Err(err) = dbus_interface.set_dns(*iface, config.resolvers.clone())
                            {
                                log::error!("Failed to re-apply DNS config - {}", err);
                            }
                            anything_changed = true;
                        }
                    }
                    if anything_changed {
                        let result = if has_only_tunnel_config(&configs, tunnel_index) {
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

        let mut initial_states = self.initial_states.lock().unwrap();
        for (iface, state) in &*initial_states {
            let result = if *iface == self.tunnel_index {
                self.dbus_interface.revert_link(state.clone()).await
            } else {
                self.dbus_interface.set_dns_state(state.clone()).await
            };
            if let Err(err) = result {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to revert interface config")
                );
            }
        }
        initial_states.clear();

        self.current_config.lock().unwrap().clear();

        Ok(())
    }
}

fn has_only_tunnel_config(configs: &BTreeMap<u32, DnsConfig>, tunnel_index: u32) -> bool {
    configs.len() == 1 && configs.contains_key(&tunnel_index)
}
