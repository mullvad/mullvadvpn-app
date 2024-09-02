//! This module spawns a thread used to service request to the split tunnel device driver.
//!
//! We've chosen isolate all dealings with the device driver on a dedicated thread as we've
//! previously faced issues with deadlocks, where other services fight us over the global
//! transaction lock in WFP (Windows Filtering Platform), among other things.
//!
//! This design makes the tunnel state machine relatively protected against driver failure.

use crate::tunnel_state_machine::TunnelCommand;
use futures::channel::mpsc;
use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{mpsc as sync_mpsc, Arc, Mutex, RwLock, Weak},
    time::Duration,
};
use talpid_types::{split_tunnel::ExcludedProcess, tunnel::ErrorStateCause, ErrorExt};

use super::{
    driver::DeviceHandle,
    path_monitor::{PathMonitor, PathMonitorHandle},
    service,
    volume_monitor::VolumeMonitor,
    Error, InterfaceAddresses,
};

const INIT_TIMEOUT: Duration = Duration::from_secs(5);

/// A request to the split tunnel monitor
pub struct Request {
    /// Request details
    details: RequestDetails,
    /// Whether to block if the request fails
    must_succeed: bool,
    /// Response channel
    response_tx: Option<sync_mpsc::Sender<Result<(), Error>>>,
}

/// The particular request to send
pub enum RequestDetails {
    /// Update paths to exclude
    SetPaths(Vec<OsString>),
    /// Update default and VPN tunnel addresses
    RegisterIps(InterfaceAddresses),
    /// Stop the split tunnel monitor
    Stop,
}

impl Request {
    pub fn new(details: RequestDetails) -> Self {
        Request {
            details,
            must_succeed: false,
            response_tx: None,
        }
    }

    pub fn response_tx(mut self, response_tx: sync_mpsc::Sender<Result<(), Error>>) -> Self {
        self.response_tx = Some(response_tx);
        self
    }

    pub fn must_succeed(mut self) -> Self {
        self.must_succeed = true;
        self
    }

    pub fn request_name(&self) -> &'static str {
        match self.details {
            RequestDetails::SetPaths(_) => "SetPaths",
            RequestDetails::RegisterIps(_) => "RegisterIps",
            RequestDetails::Stop => "Stop",
        }
    }
}

/// Begin servicing requests sent on the returned channel
pub fn spawn_request_thread(
    resource_dir: PathBuf,
    daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
    volume_update_rx: mpsc::UnboundedReceiver<()>,
    excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
) -> Result<(sync_mpsc::Sender<Request>, Arc<DeviceHandle>), Error> {
    let (tx, rx): (sync_mpsc::Sender<Request>, _) = sync_mpsc::channel();
    let (init_tx, init_rx) = sync_mpsc::channel();

    let monitored_paths = Arc::new(Mutex::new(vec![]));
    let monitored_paths_copy = monitored_paths.clone();

    let (monitor_tx, monitor_rx) = sync_mpsc::channel();

    let path_monitor = PathMonitor::spawn(monitor_tx.clone()).map_err(Error::StartPathMonitor)?;
    let volume_monitor = VolumeMonitor::spawn(
        path_monitor.clone(),
        monitor_tx,
        monitored_paths.clone(),
        volume_update_rx,
    );

    std::thread::spawn(move || {
        // Ensure that the device driver service is running and that we have a handle to it
        let handle = match setup_and_create_device(&resource_dir) {
            Ok(handle) => {
                let _ = init_tx.send(Ok(handle.clone()));
                handle
            }
            Err(error) => {
                let _ = init_tx.send(Err(error));
                return;
            }
        };

        request_loop(
            handle.clone(),
            rx,
            daemon_tx,
            monitored_paths,
            path_monitor.clone(),
            excluded_processes,
        );

        // Shut components down in a sane order
        drop(volume_monitor);
        if let Err(error) = path_monitor.shutdown() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to shut down path monitor")
            );
        }

        // The device handle must be dropped before stopping the service
        debug_assert_eq!(Arc::strong_count(&handle), 1);
        drop(handle);

        log::debug!("Stopping ST service");
        if let Err(error) = service::stop_driver_service() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to stop ST service")
            );
        }
    });

    let handle = init_rx
        .recv_timeout(INIT_TIMEOUT)
        .map_err(|_| Error::RequestThreadStuck)??;

    let handle_copy = handle.clone();

    std::thread::spawn(move || {
        while let Ok(()) = monitor_rx.recv() {
            let paths = monitored_paths_copy.lock().unwrap();
            let result = if paths.len() > 0 {
                log::debug!("Re-resolving excluded paths");
                handle_copy.set_config(&paths)
            } else {
                continue;
            };
            if let Err(error) = result {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to update excluded paths")
                );
            }
        }
    });

    Ok((tx, handle))
}

/// Install the driver and open a handle for it
fn setup_and_create_device(resource_dir: &Path) -> Result<Arc<DeviceHandle>, Error> {
    service::install_driver_if_required(resource_dir).map_err(Error::ServiceError)?;
    DeviceHandle::new()
        .map(Arc::new)
        .map_err(Error::InitializationError)
}

/// Service requests to the device driver
fn request_loop(
    handle: Arc<DeviceHandle>,
    cmd_rx: sync_mpsc::Receiver<Request>,
    daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
    monitored_paths: Arc<Mutex<Vec<OsString>>>,
    path_monitor: PathMonitorHandle,
    excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
) {
    let mut previous_addresses = InterfaceAddresses::default();

    while let Ok(request) = cmd_rx.recv() {
        let request_name = request.request_name();

        let (should_stop, response) = handle_request(
            request.details,
            &handle,
            &path_monitor,
            &monitored_paths,
            &excluded_processes,
            &mut previous_addresses,
        );

        handle_request_result(
            &daemon_tx,
            response,
            request.must_succeed,
            request_name,
            request.response_tx,
        );

        // Stop request loop
        if should_stop {
            break;
        }
    }
}

/// Handle a request to the split tunnel device
fn handle_request(
    request: RequestDetails,
    handle: &DeviceHandle,
    path_monitor: &PathMonitorHandle,
    monitored_paths: &Arc<Mutex<Vec<OsString>>>,
    excluded_processes: &Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    previous_addresses: &mut InterfaceAddresses,
) -> (bool, Result<(), Error>) {
    let (should_stop, result) = match request {
        RequestDetails::SetPaths(paths) => {
            let mut monitored_paths_guard = monitored_paths.lock().unwrap();

            let result = if !paths.is_empty() {
                handle.set_config(&paths).map_err(Error::SetConfiguration)
            } else {
                handle.clear_config().map_err(Error::SetConfiguration)
            };

            if result.is_ok() {
                if let Err(error) = path_monitor.set_paths(&paths) {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to update path monitor")
                    );
                }
                *monitored_paths_guard = paths.to_vec();
            }

            (false, result)
        }
        RequestDetails::RegisterIps(mut ips) => {
            if ips.internet_ipv4.is_none() && ips.internet_ipv6.is_none() {
                ips.tunnel_ipv4 = None;
                ips.tunnel_ipv6 = None;
            }
            if previous_addresses == &ips {
                (false, Ok(()))
            } else {
                let result = handle
                    .register_ips(
                        ips.tunnel_ipv4,
                        ips.tunnel_ipv6,
                        ips.internet_ipv4,
                        ips.internet_ipv6,
                    )
                    .map_err(Error::RegisterIps);
                if result.is_ok() {
                    *previous_addresses = ips;
                }
                (false, result)
            }
        }
        RequestDetails::Stop => {
            if let Err(error) = handle.reset().map_err(Error::ResetError) {
                // Shut down failed, so continue to live
                return (false, Err(error));
            }

            // Clean up
            monitored_paths.lock().unwrap().clear();
            excluded_processes.write().unwrap().clear();

            // Stop listening to commands
            (true, Ok(()))
        }
    };

    (should_stop, result)
}

/// Handle the result of a request
fn handle_request_result(
    daemon_tx: &Weak<mpsc::UnboundedSender<TunnelCommand>>,
    result: Result<(), Error>,
    must_succeed: bool,
    request_name: &str,
    response_tx: Option<sync_mpsc::Sender<Result<(), Error>>>,
) {
    let log_request_failure = |response: &Result<(), Error>| {
        if let Err(error) = response {
            log::error!(
                "Request/ioctl failed: {}\n{}",
                request_name,
                error.display_chain()
            );
        }
    };

    let request_failed = result.is_err();

    if let Some(response_tx) = response_tx {
        if let Err(error) = response_tx.send(result) {
            log::error!(
                "A response could not be sent for completed request/ioctl: {}",
                request_name
            );
            log_request_failure(&error.0);
        }
    } else {
        log_request_failure(&result);
    }

    // Move to error state if the request failed but must succeed
    if request_failed && must_succeed {
        if let Some(daemon_tx) = daemon_tx.upgrade() {
            log::debug!(
                "Entering error state due to failed request/ioctl: {}",
                request_name
            );
            let _ =
                daemon_tx.unbounded_send(TunnelCommand::Block(ErrorStateCause::SplitTunnelError));
        } else {
            log::error!("Cannot handle failed request since tunnel state machine is down");
        }
    }
}
