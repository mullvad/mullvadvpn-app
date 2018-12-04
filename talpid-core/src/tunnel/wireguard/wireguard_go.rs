use std::{
    ffi::CString,
    fs,
    os::unix::io::AsRawFd,
    path::Path,
    sync::{Arc, Condvar, Mutex},
};

use super::{
    super::super::network_interface::{NetworkInterface, TunnelDevice},
    CloseHandle, Config, Error, ErrorKind, Result, ResultExt, Tunnel,
};

type WgLogLevel = i32;
// wireguard-go supports log levels 0 through 3 with 3 being the most verbose
const WG_GO_LOG_DEBUG: WgLogLevel = 3;


pub struct WgGoTunnel {
    handle: WgHandle,
    interface_name: String,
    // keeping ahold of the tunnel device and the log file ensures that
    // the associated file handles end up closed
    _tunnel_device: TunnelDevice,
    _log_file: fs::File,
}

impl WgGoTunnel {
    pub fn start_tunnel(config: &Config, log_path: Option<&Path>) -> Result<Self> {
        let mut tunnel_device =
            TunnelDevice::new().chain_err(|| ErrorKind::SetupTunnelDeviceError)?;

        for ip in config.interface.addresses.iter() {
            tunnel_device
                .set_ip(*ip)
                .chain_err(|| ErrorKind::SetupTunnelDeviceError)?;
        }

        tunnel_device
            .set_up(true)
            .chain_err(|| ErrorKind::SetupTunnelDeviceError)?;

        let interface_name: String = tunnel_device.get_name().to_string();
        let fd = tunnel_device.as_raw_fd();
        let log_file = prepare_log_file(log_path)?;
        let handle = WgHandle::new(
            &interface_name,
            config,
            fd,
            log_file.as_raw_fd(),
            WG_GO_LOG_DEBUG,
        )?;

        Ok(WgGoTunnel {
            handle,
            interface_name,
            _tunnel_device: tunnel_device,
            _log_file: log_file,
        })
    }
}

fn prepare_log_file(base_path: Option<&Path>) -> Result<fs::File> {
    match base_path {
        Some(path) => {
            if path.exists() {
                fs::rename(&path, &path.with_extension(".old"))
                    .chain_err(|| ErrorKind::PrepareLogFileError)?;
            }
            let file = fs::File::create(&path).chain_err(|| ErrorKind::PrepareLogFileError)?;
            Ok(file)
        }
        None => {
            let log_file =
                fs::File::open("/dev/null").chain_err(|| ErrorKind::PrepareLogFileError)?;
            Ok(log_file)
        }
    }
}

impl Tunnel for WgGoTunnel {
    fn close_handle(&self) -> Box<dyn CloseHandle> {
        Box::new(self.handle.clone())
    }

    fn get_interface_name(&self) -> &str {
        &self.interface_name
    }

    fn wait(mut self: Box<Self>) -> Result<()> {
        self.handle.wait()
    }
}

#[derive(Clone)]
struct WgHandle {
    handle: Arc<(Mutex<WgHandleInner>, Condvar)>,
}

impl WgHandle {
    fn new(
        interface_name: &str,
        config: &Config,
        fd: handle::Descriptor,
        log_fd: handle::Descriptor,
        log_level: WgLogLevel,
    ) -> Result<WgHandle> {
        let inner = WgHandleInner::new(interface_name, config, fd, log_fd, log_level)?;
        Ok(Self {
            handle: Arc::new((Mutex::new(inner), Condvar::new())),
        })
    }

    fn wait(&mut self) -> Result<()> {
        let (mutex, condition) = self.handle.as_ref();
        let mut handle = mutex
            .lock()
            .expect("wireguard-go handle mutex got poisoned");
        loop {
            // checking if handle has been removed
            if handle.has_stopped() {
                return handle.consume_error();
            }

            handle = condition
                .wait(handle)
                .expect("wireguard-go handle mutex got poisoned");
        }
    }
}

impl CloseHandle for WgHandle {
    fn close(&mut self) {
        let (mutex, condition) = &self.handle.as_ref();
        mutex
            .lock()
            .expect("wireguard-go handle mutex got poisoned")
            .close();
        condition.notify_all();
    }

    fn close_with_error(&mut self, err: Error) {
        let (mutex, condition) = &self.handle.as_ref();
        mutex
            .lock()
            .expect("wireguard-go handle mutex got poisoned")
            .close_with_error(err);
        condition.notify_all();
    }
}

#[cfg(unix)]
mod handle {
    pub type Descriptor = std::os::unix::io::RawFd;
}

#[cfg(windows)]
mod handle {
    pub type Descriptor = std::os::windows::io::RawHandle;
}


#[link(name = "wg", kind = "static")]
extern "C" {
    fn wgTurnOnWithFd(
        iface_name: *const i8,
        mtu: i64,
        settings: *const i8,
        fd: handle::Descriptor,
        log_fd: handle::Descriptor,
        logLevel: WgLogLevel,
    ) -> i32;
    fn wgTurnOff(handle: i32) -> i32;
}


struct WgHandleInner {
    handle: Option<i32>,
    error: Option<Error>,
}

impl WgHandleInner {
    fn new(
        interface_name: &str,
        config: &Config,
        fd: handle::Descriptor,
        log_fd: handle::Descriptor,
        log_level: WgLogLevel,
    ) -> Result<WgHandleInner> {
        let wg_config_str = CString::new(config.get_wg_config()).expect("Found \\0 in wg config");
        let iface_name =
            CString::new(interface_name.as_bytes()).expect("Found \\0 in interface name");
        let handle = unsafe {
            wgTurnOnWithFd(
                iface_name.as_ptr(),
                config.interface.mtu as i64,
                wg_config_str.as_ptr(),
                fd,
                log_fd,
                log_level,
            )
        };
        if handle < 0 {
            bail!(ErrorKind::StartWireguardError(handle));
        };
        Ok(Self {
            handle: Some(handle),
            error: None,
        })
    }

    fn has_stopped(&self) -> bool {
        self.handle.is_none()
    }

    fn consume_error(&mut self) -> Result<()> {
        self.error.take().map(Err).unwrap_or(Ok(()))
    }

    fn close(&mut self) {
        if let Some(handle) = self.handle.take() {
            let status = unsafe { wgTurnOff(handle) };
            if status < 0 {
                if self.error.is_none() {
                    self.error = Some(ErrorKind::StopWireguardError(status).into());
                } else {
                    log::error!(
                        "Failed to stop wireguard-go successfully - {}",
                        ErrorKind::StopWireguardError(status)
                    );
                }
            };
        }
    }

    fn close_with_error(&mut self, err: Error) {
        // if we're already shut down, there's nothing to do
        if self.has_stopped() {
            return;
        }

        // if there's an error that's already set, there's not much to do here.
        if let Some(set_err) = self.error.as_ref() {
            log::trace!(
                "Wireguard handle error already set - '{}', dropping another one - {}",
                set_err,
                err
            );
            return;
        }

        self.error = Some(err);
        self.close();
    }
}

impl Drop for WgHandleInner {
    fn drop(&mut self) {
        if let Some(handle) = self.handle {
            let status = unsafe { wgTurnOff(handle) };
            if status < 0 {
                log::trace!(
                    "Wireguard tunnel returned non-zero status when shutting down - {}",
                    status
                );
            }
        }
    }
}
