mod driver;
mod path_monitor;
mod service;
mod volume_monitor;
mod windows;

use crate::{tunnel::TunnelMetadata, tunnel_state_machine::TunnelCommand};
use futures::channel::{mpsc, oneshot};
use std::{
    collections::HashMap,
    convert::TryFrom,
    ffi::{OsStr, OsString},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc as sync_mpsc, Arc, Mutex, MutexGuard, RwLock, Weak,
    },
    time::Duration,
};
use talpid_routing::{get_best_default_route, CallbackHandle, EventType, RouteManagerHandle};
use talpid_types::{tunnel::ErrorStateCause, ErrorExt};
use talpid_windows_net::{get_ip_address_for_interface, AddressFamily};
use windows_sys::Win32::Foundation::ERROR_OPERATION_ABORTED;

const DRIVER_EVENT_BUFFER_SIZE: usize = 2048;
const RESERVED_IP_V4: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 123);

/// Errors that may occur in [`SplitTunnel`].
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to install or start driver service
    #[error(display = "Failed to start driver service")]
    ServiceError(#[error(source)] service::Error),

    /// Failed to initialize the driver
    #[error(display = "Failed to initialize driver")]
    InitializationError(#[error(source)] driver::DeviceHandleError),

    /// Failed to reset the driver
    #[error(display = "Failed to reset driver")]
    ResetError(#[error(source)] io::Error),

    /// Failed to set paths to excluded applications
    #[error(display = "Failed to set list of excluded applications")]
    SetConfiguration(#[error(source)] io::Error),

    /// Failed to obtain the current driver state
    #[error(display = "Failed to obtain the driver state")]
    GetState(#[error(source)] io::Error),

    /// Failed to register interface IP addresses
    #[error(display = "Failed to register IP addresses for exclusions")]
    RegisterIps(#[error(source)] io::Error),

    /// Failed to clear interface IP addresses
    #[error(display = "Failed to clear registered IP addresses")]
    ClearIps(#[error(source)] io::Error),

    /// Failed to set up the driver event loop
    #[error(display = "Failed to set up the driver event loop")]
    EventThreadError(#[error(source)] io::Error),

    /// Failed to obtain default route
    #[error(display = "Failed to obtain the default route")]
    ObtainDefaultRoute(#[error(source)] talpid_routing::Error),

    /// Failed to obtain an IP address given a network interface LUID
    #[error(display = "Failed to obtain IP address for interface LUID")]
    LuidToIp(#[error(source)] talpid_windows_net::Error),

    /// Failed to set up callback for monitoring default route changes
    #[error(display = "Failed to register default route change callback")]
    RegisterRouteChangeCallback,

    /// Unexpected IP parsing error
    #[error(display = "Failed to parse IP address")]
    IpParseError,

    /// The request handling thread is stuck
    #[error(display = "The ST request thread is stuck")]
    RequestThreadStuck,

    /// The request handling thread is down
    #[error(display = "The split tunnel monitor is down")]
    SplitTunnelDown,

    /// Failed to start the NTFS reparse point monitor
    #[error(display = "Failed to start path monitor")]
    StartPathMonitor(#[error(source)] io::Error),

    /// A previous path update has not yet completed
    #[error(display = "A previous update is not yet complete")]
    AlreadySettingPaths,

    /// Resetting in the engaged state risks leaking into the tunnel
    #[error(display = "Failed to reset driver because it is engaged")]
    CannotResetEngaged,
}

/// Manages applications whose traffic to exclude from the tunnel.
pub struct SplitTunnel {
    runtime: tokio::runtime::Handle,
    request_tx: RequestTx,
    event_thread: Option<std::thread::JoinHandle<()>>,
    quit_event: Arc<windows::Event>,
    excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    _route_change_callback: Option<CallbackHandle>,
    daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
    async_path_update_in_progress: Arc<AtomicBool>,
    route_manager: RouteManagerHandle,
}

enum Request {
    SetPaths(Vec<OsString>),
    RegisterIps(InterfaceAddresses),
    Stop,
}
type RequestResponseTx = sync_mpsc::Sender<Result<(), Error>>;
type RequestTx = sync_mpsc::Sender<(Request, RequestResponseTx)>;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default, PartialEq, Clone)]
struct InterfaceAddresses {
    tunnel_ipv4: Option<Ipv4Addr>,
    tunnel_ipv6: Option<Ipv6Addr>,
    internet_ipv4: Option<Ipv4Addr>,
    internet_ipv6: Option<Ipv6Addr>,
}

/// Represents a process that is being excluded from the tunnel.
#[derive(Debug, Clone)]
pub struct ExcludedProcess {
    /// Process identifier.
    pub pid: u32,
    /// Path to the image that this process is an instance of.
    pub image: PathBuf,
    /// If true, then the process is split because its parent was split,
    /// not due to its path being in the config.
    pub inherited: bool,
}

/// Cloneable handle for interacting with the split tunnel module.
#[derive(Debug, Clone)]
pub struct SplitTunnelHandle {
    excluded_processes: Weak<RwLock<HashMap<usize, ExcludedProcess>>>,
}

impl SplitTunnelHandle {
    /// Return processes that are currently being excluded, including
    /// their pids, paths, and reason for being excluded.
    pub fn get_processes(&self) -> Result<Vec<ExcludedProcess>, Error> {
        let processes = self
            .excluded_processes
            .upgrade()
            .ok_or(Error::SplitTunnelDown)?;
        let processes = processes.read().unwrap();
        Ok(processes.values().cloned().collect())
    }
}

enum EventResult {
    /// Result containing the next event.
    Event(driver::EventId, driver::EventBody),
    /// Quit event was signaled.
    Quit,
}

impl SplitTunnel {
    /// Initialize the split tunnel device.
    pub fn new(
        runtime: tokio::runtime::Handle,
        resource_dir: PathBuf,
        daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
        volume_update_rx: mpsc::UnboundedReceiver<()>,
        route_manager: RouteManagerHandle,
    ) -> Result<Self, Error> {
        let excluded_processes = Arc::new(RwLock::new(HashMap::new()));

        let (request_tx, handle) =
            Self::spawn_request_thread(resource_dir, volume_update_rx, excluded_processes.clone())?;

        let (event_thread, quit_event) =
            Self::spawn_event_listener(handle, excluded_processes.clone())?;

        Ok(SplitTunnel {
            runtime,
            request_tx,
            event_thread: Some(event_thread),
            quit_event,
            _route_change_callback: None,
            daemon_tx,
            async_path_update_in_progress: Arc::new(AtomicBool::new(false)),
            excluded_processes,
            route_manager,
        })
    }

    /// Spawns an event loop thread that processes events from the driver service.
    fn spawn_event_listener(
        handle: Arc<driver::DeviceHandle>,
        excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) -> Result<(std::thread::JoinHandle<()>, Arc<windows::Event>), Error> {
        let mut event_overlapped = windows::Overlapped::new(Some(
            windows::Event::new(true, false).map_err(Error::EventThreadError)?,
        ))
        .map_err(Error::EventThreadError)?;

        let quit_event =
            Arc::new(windows::Event::new(true, false).map_err(Error::EventThreadError)?);
        let quit_event_copy = quit_event.clone();

        let event_thread = std::thread::spawn(move || {
            log::debug!("Starting split tunnel event thread");
            let mut data_buffer = vec![];

            loop {
                // Wait until either the next event is received or the quit event is signaled.
                let (event_id, event_body) = match Self::fetch_next_event(
                    &handle,
                    &quit_event,
                    &mut event_overlapped,
                    &mut data_buffer,
                ) {
                    Ok(EventResult::Event(event_id, event_body)) => (event_id, event_body),
                    Ok(EventResult::Quit) => break,
                    Err(error) => {
                        if error.raw_os_error() == Some(ERROR_OPERATION_ABORTED as i32) {
                            // The driver will normally abort the request if the driver state
                            // is reset. Give the driver service some time to recover before
                            // retrying.
                            std::thread::sleep(Duration::from_millis(500));
                        }
                        continue;
                    }
                };

                Self::handle_event(event_id, event_body, &excluded_processes);
            }

            log::debug!("Stopping split tunnel event thread");
        });

        Ok((event_thread, quit_event_copy))
    }

    fn fetch_next_event(
        device: &Arc<driver::DeviceHandle>,
        quit_event: &windows::Event,
        overlapped: &mut windows::Overlapped,
        data_buffer: &mut Vec<u8>,
    ) -> io::Result<EventResult> {
        if unsafe { driver::wait_for_single_object(quit_event.as_handle(), Some(Duration::ZERO)) }
            .is_ok()
        {
            return Ok(EventResult::Quit);
        }

        data_buffer.resize(DRIVER_EVENT_BUFFER_SIZE, 0u8);

        unsafe {
            driver::device_io_control_buffer_async(
                device,
                driver::DriverIoctlCode::DequeEvent as u32,
                None,
                data_buffer.as_mut_ptr(),
                u32::try_from(data_buffer.len()).expect("buffer must be smaller than u32"),
                overlapped.as_mut_ptr(),
            )
        }
        .map_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("DeviceIoControl failed to deque event")
            );
            error
        })?;

        let event_objects = [
            overlapped.get_event().unwrap().as_handle(),
            quit_event.as_handle(),
        ];

        let signaled_object =
            unsafe { driver::wait_for_multiple_objects(&event_objects[..], false) }.map_err(
                |error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("wait_for_multiple_objects failed")
                    );
                    error
                },
            )?;

        if signaled_object == quit_event.as_handle() {
            // Quit event was signaled
            return Ok(EventResult::Quit);
        }

        let returned_bytes =
            driver::get_overlapped_result(device, overlapped).map_err(|error| {
                if error.raw_os_error() != Some(ERROR_OPERATION_ABORTED as i32) {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "get_overlapped_result failed for dequed event"
                        ),
                    );
                }
                error
            })?;

        data_buffer
            .truncate(usize::try_from(returned_bytes).expect("usize must be no smaller than u32"));

        driver::parse_event_buffer(&data_buffer)
            .map(|(id, body)| EventResult::Event(id, body))
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to parse ST event buffer")
                );
                io::Error::new(io::ErrorKind::Other, "Failed to parse ST event buffer")
            })
    }

    fn handle_event(
        event_id: driver::EventId,
        event_body: driver::EventBody,
        excluded_processes: &Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) {
        use driver::{EventBody, EventId};

        let event_str = match &event_id {
            EventId::StartSplittingProcess | EventId::ErrorStartSplittingProcess => {
                "Start splitting process"
            }
            EventId::StopSplittingProcess | EventId::ErrorStopSplittingProcess => {
                "Stop splitting process"
            }
            EventId::ErrorMessage => "ErrorMessage",
        };

        match event_body {
            EventBody::SplittingEvent {
                process_id,
                reason,
                image,
            } => {
                let mut pids = excluded_processes.write().unwrap();
                match event_id {
                    EventId::StartSplittingProcess => {
                        if let Some(prev_entry) = pids.get(&process_id) {
                            log::error!("PID collision: {process_id} is already in the list of excluded processes. New image: {:?}. Current image: {:?}", image, prev_entry);
                        }
                        pids.insert(
                            process_id,
                            ExcludedProcess {
                                pid: u32::try_from(process_id)
                                    .expect("PID should be containable in a DWORD"),
                                image: Path::new(&image).to_path_buf(),
                                inherited: reason
                                    .contains(driver::SplittingChangeReason::BY_INHERITANCE),
                            },
                        );
                    }
                    EventId::StopSplittingProcess => {
                        if pids.remove(&process_id).is_none() {
                            log::error!("Inconsistent process tree: {process_id} was not found");
                        }
                    }
                    _ => (),
                }

                log::trace!(
                    "{}:\n\tpid: {}\n\treason: {:?}\n\timage: {:?}",
                    event_str,
                    process_id,
                    reason,
                    image,
                );
            }
            EventBody::SplittingError { process_id, image } => {
                log::error!(
                    "FAILED: {}:\n\tpid: {}\n\timage: {:?}",
                    event_str,
                    process_id,
                    image,
                );
            }
            EventBody::ErrorMessage { status, message } => {
                log::error!("NTSTATUS {:#x}: {}", status, message.to_string_lossy())
            }
        }
    }

    fn spawn_request_thread(
        resource_dir: PathBuf,
        volume_update_rx: mpsc::UnboundedReceiver<()>,
        excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) -> Result<(RequestTx, Arc<driver::DeviceHandle>), Error> {
        let (tx, rx): (RequestTx, _) = sync_mpsc::channel();
        let (init_tx, init_rx) = sync_mpsc::channel();

        let monitored_paths = Arc::new(Mutex::new(vec![]));
        let monitored_paths_copy = monitored_paths.clone();

        let (monitor_tx, monitor_rx) = sync_mpsc::channel();

        let path_monitor = path_monitor::PathMonitor::spawn(monitor_tx.clone())
            .map_err(Error::StartPathMonitor)?;
        let volume_monitor = volume_monitor::VolumeMonitor::spawn(
            path_monitor.clone(),
            monitor_tx,
            monitored_paths.clone(),
            volume_update_rx,
        );

        std::thread::spawn(move || {
            let init_fn = || {
                service::install_driver_if_required(&resource_dir).map_err(Error::ServiceError)?;
                driver::DeviceHandle::new()
                    .map(Arc::new)
                    .map_err(Error::InitializationError)
            };

            let handle = match init_fn() {
                Ok(handle) => {
                    let _ = init_tx.send(Ok(handle.clone()));
                    handle
                }
                Err(error) => {
                    let _ = init_tx.send(Err(error));
                    return;
                }
            };

            let mut previous_addresses = InterfaceAddresses::default();

            while let Ok((request, response_tx)) = rx.recv() {
                let response = match request {
                    Request::SetPaths(paths) => {
                        let mut monitored_paths_guard = monitored_paths.lock().unwrap();

                        let result = if paths.len() > 0 {
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

                        result
                    }
                    Request::RegisterIps(mut ips) => {
                        if ips.internet_ipv4.is_none() && ips.internet_ipv6.is_none() {
                            ips.tunnel_ipv4 = None;
                            ips.tunnel_ipv6 = None;
                        }
                        if previous_addresses == ips {
                            Ok(())
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
                                previous_addresses = ips;
                            }
                            result
                        }
                    }
                    Request::Stop => {
                        if let Err(error) = handle.reset().map_err(Error::ResetError) {
                            let _ = response_tx.send(Err(error));
                            continue;
                        }

                        monitored_paths.lock().unwrap().clear();
                        excluded_processes.write().unwrap().clear();

                        let _ = response_tx.send(Ok(()));

                        // Stop listening to commands
                        break;
                    }
                };
                if response_tx.send(response).is_err() {
                    log::error!("A response could not be sent for a completed request");
                }
            }

            drop(volume_monitor);
            if let Err(error) = path_monitor.shutdown() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to shut down path monitor")
                );
            }

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
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| Error::RequestThreadStuck)??;

        let handle_copy = handle.clone();

        std::thread::spawn(move || {
            while let Ok(()) = monitor_rx.recv() {
                let paths = monitored_paths_copy.lock().unwrap();
                let result = if paths.len() > 0 {
                    log::debug!("Re-resolving excluded paths");
                    handle_copy.set_config(&*paths)
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

    fn send_request(&self, request: Request) -> Result<(), Error> {
        Self::send_request_inner(&self.request_tx, request)
    }

    fn send_request_inner(request_tx: &RequestTx, request: Request) -> Result<(), Error> {
        let (response_tx, response_rx) = sync_mpsc::channel();

        request_tx
            .send((request, response_tx))
            .map_err(|_| Error::SplitTunnelDown)?;

        response_rx
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| Error::RequestThreadStuck)?
    }

    /// Set a list of applications to exclude from the tunnel.
    pub fn set_paths_sync<T: AsRef<OsStr>>(&self, paths: &[T]) -> Result<(), Error> {
        self.send_request(Request::SetPaths(
            paths
                .iter()
                .map(|path| path.as_ref().to_os_string())
                .collect(),
        ))
    }

    /// Set a list of applications to exclude from the tunnel.
    pub fn set_paths<T: AsRef<OsStr>>(
        &self,
        paths: &[T],
        result_tx: oneshot::Sender<Result<(), Error>>,
    ) {
        let busy = self
            .async_path_update_in_progress
            .swap(true, Ordering::SeqCst);
        if busy {
            let _ = result_tx.send(Err(Error::AlreadySettingPaths));
            return;
        }
        let (response_tx, response_rx) = sync_mpsc::channel();
        let request = Request::SetPaths(
            paths
                .iter()
                .map(|path| path.as_ref().to_os_string())
                .collect(),
        );
        let request_tx = self.request_tx.clone();

        let wait_task = move || {
            request_tx
                .send((request, response_tx))
                .map_err(|_| Error::SplitTunnelDown)?;
            response_rx.recv().map_err(|_| Error::SplitTunnelDown)?
        };
        let in_progress = self.async_path_update_in_progress.clone();
        self.runtime.spawn_blocking(move || {
            let _ = result_tx.send(wait_task());
            in_progress.store(false, Ordering::SeqCst);
        });
    }

    /// Instructs the driver to redirect traffic from sockets bound to 0.0.0.0, ::, or the
    /// tunnel addresses (if any) to the default route.
    pub fn set_tunnel_addresses(&mut self, metadata: Option<&TunnelMetadata>) -> Result<(), Error> {
        let mut tunnel_ipv4 = None;
        let mut tunnel_ipv6 = None;

        if let Some(metadata) = metadata {
            for ip in &metadata.ips {
                match ip {
                    IpAddr::V4(address) => tunnel_ipv4 = Some(address.clone()),
                    IpAddr::V6(address) => tunnel_ipv6 = Some(address.clone()),
                }
            }
        }

        let tunnel_ipv4 = Some(tunnel_ipv4.unwrap_or(RESERVED_IP_V4));
        let context_mutex = Arc::new(Mutex::new(
            SplitTunnelDefaultRouteChangeHandlerContext::new(
                self.request_tx.clone(),
                self.daemon_tx.clone(),
                tunnel_ipv4,
                tunnel_ipv6,
            ),
        ));

        self._route_change_callback = None;
        let moved_context_mutex = context_mutex.clone();
        let context = context_mutex.lock().unwrap();
        let callback = self
            .runtime
            .block_on(
                self.route_manager
                    .add_default_route_change_callback(Box::new(move |event, addr_family| {
                        split_tunnel_default_route_change_handler(
                            event,
                            addr_family,
                            &moved_context_mutex,
                        )
                    })),
            )
            .map(Some)
            // NOTE: This cannot fail if a callback is created. If that assumption is wrong, this
            // could deadlock if the dropped callback is invoked (see `init_context`).
            .map_err(|_| Error::RegisterRouteChangeCallback)?;

        Self::init_context(context)?;
        self._route_change_callback = callback;

        Ok(())
    }

    fn init_context(
        mut context: MutexGuard<'_, SplitTunnelDefaultRouteChangeHandlerContext>,
    ) -> Result<(), Error> {
        // NOTE: This should remain a separate function. Dropping the context after `callback`
        // causes a deadlock if `split_tunnel_default_route_change_handler` is called at the same
        // time (i.e. if a route change has occurred), since it waits on the context and
        // `CallbackHandle::drop` also waits for `split_tunnel_default_route_change_handler`
        // to complete.

        context.initialize_internet_addresses()?;
        context.register_ips()
    }

    /// Instructs the driver to stop redirecting tunnel traffic and INADDR_ANY.
    pub fn clear_tunnel_addresses(&mut self) -> Result<(), Error> {
        self._route_change_callback = None;
        self.send_request(Request::RegisterIps(InterfaceAddresses::default()))
    }

    /// Returns a handle used for interacting with the split tunnel module.
    pub fn handle(&self) -> SplitTunnelHandle {
        SplitTunnelHandle {
            excluded_processes: Arc::downgrade(&self.excluded_processes),
        }
    }
}

impl Drop for SplitTunnel {
    fn drop(&mut self) {
        if let Some(_event_thread) = self.event_thread.take() {
            if let Err(error) = self.quit_event.set() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to close ST event thread")
                );
            }
            // Not joining `event_thread`: It may be unresponsive.
        }

        if let Err(error) = self.send_request(Request::Stop) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to stop ST driver service")
            );
        }
    }
}

struct SplitTunnelDefaultRouteChangeHandlerContext {
    request_tx: RequestTx,
    pub daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
    pub addresses: InterfaceAddresses,
}

impl SplitTunnelDefaultRouteChangeHandlerContext {
    pub fn new(
        request_tx: RequestTx,
        daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
        tunnel_ipv4: Option<Ipv4Addr>,
        tunnel_ipv6: Option<Ipv6Addr>,
    ) -> Self {
        SplitTunnelDefaultRouteChangeHandlerContext {
            request_tx,
            daemon_tx,
            addresses: InterfaceAddresses {
                tunnel_ipv4,
                tunnel_ipv6,
                internet_ipv4: None,
                internet_ipv6: None,
            },
        }
    }

    pub fn register_ips(&self) -> Result<(), Error> {
        SplitTunnel::send_request_inner(
            &self.request_tx,
            Request::RegisterIps(self.addresses.clone()),
        )
    }

    pub fn initialize_internet_addresses(&mut self) -> Result<(), Error> {
        // Identify IP address that gives us Internet access
        let internet_ipv4 = get_best_default_route(AddressFamily::Ipv4)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| {
                get_ip_address_for_interface(AddressFamily::Ipv4, route.iface).map(|ip| match ip {
                    Some(IpAddr::V4(addr)) => Some(addr),
                    Some(_) => unreachable!("wrong address family (expected IPv4)"),
                    None => {
                        log::warn!("No IPv4 address was found for the default route interface");
                        None
                    }
                })
            })
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();
        let internet_ipv6 = get_best_default_route(AddressFamily::Ipv6)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| {
                get_ip_address_for_interface(AddressFamily::Ipv6, route.iface).map(|ip| match ip {
                    Some(IpAddr::V6(addr)) => Some(addr),
                    Some(_) => unreachable!("wrong address family (expected IPv6)"),
                    None => {
                        log::warn!("No IPv6 address was found for the default route interface");
                        None
                    }
                })
            })
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();

        self.addresses.internet_ipv4 = internet_ipv4;
        self.addresses.internet_ipv6 = internet_ipv6;

        Ok(())
    }
}

fn split_tunnel_default_route_change_handler<'a>(
    event_type: EventType<'a>,
    address_family: AddressFamily,
    ctx_mutex: &Arc<Mutex<SplitTunnelDefaultRouteChangeHandlerContext>>,
) {
    use talpid_routing::EventType::*;

    // Update the "internet interface" IP when best default route changes
    let mut ctx = ctx_mutex.lock().expect("ST route handler mutex poisoned");

    let daemon_tx = ctx.daemon_tx.upgrade();
    let maybe_send = move |content| {
        if let Some(tx) = daemon_tx {
            let _ = tx.unbounded_send(content);
        }
    };

    let result = match event_type {
        Updated(default_route) | UpdatedDetails(default_route) => {
            match get_ip_address_for_interface(address_family, default_route.iface) {
                Ok(Some(ip)) => match IpAddr::from(ip) {
                    IpAddr::V4(addr) => ctx.addresses.internet_ipv4 = Some(addr),
                    IpAddr::V6(addr) => ctx.addresses.internet_ipv6 = Some(addr),
                },
                Ok(None) => {
                    log::warn!("Failed to obtain default route interface address");
                    match address_family {
                        AddressFamily::Ipv4 => {
                            ctx.addresses.internet_ipv4 = None;
                        }
                        AddressFamily::Ipv6 => {
                            ctx.addresses.internet_ipv6 = None;
                        }
                    }
                }
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to obtain default route interface address"
                        )
                    );
                    maybe_send(TunnelCommand::Block(ErrorStateCause::SplitTunnelError));
                    return;
                }
            };

            ctx.register_ips()
        }
        // no default route
        Removed => {
            match address_family {
                AddressFamily::Ipv4 => {
                    ctx.addresses.internet_ipv4 = None;
                }
                AddressFamily::Ipv6 => {
                    ctx.addresses.internet_ipv6 = None;
                }
            }
            ctx.register_ips()
        }
    };

    if let Err(error) = result {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to register new addresses in split tunnel driver")
        );
        maybe_send(TunnelCommand::Block(ErrorStateCause::SplitTunnelError));
    }
}
