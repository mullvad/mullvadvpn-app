use super::{
    stats::{Stats, StatsMap},
    Config, Tunnel, TunnelError,
};
use crate::logging::{clean_up_logging, initialize_logging};
use ipnetwork::IpNetwork;
use std::{
    ffi::{c_char, c_void, CStr},
    future::Future,
    path::Path,
    pin::Pin,
};
use talpid_tunnel::tun_provider::TunProvider;
use talpid_types::BoxedError;
use zeroize::Zeroize;

#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider;

use std::{
    net::IpAddr,
    os::unix::io::{AsRawFd, RawFd},
};
use talpid_tunnel::tun_provider::{Tun, TunConfig};

type Result<T> = std::result::Result<T, TunnelError>;

use std::sync::{Arc, Mutex};

const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

struct LoggingContext(u32);

impl Drop for LoggingContext {
    fn drop(&mut self) {
        clean_up_logging(self.0);
    }
}

pub struct WgGoTunnel {
    interface_name: String,
    handle: Option<i32>,
    // holding on to the tunnel device and the log file ensures that the associated file handles
    // live long enough and get closed when the tunnel is stopped
    _tunnel_device: Tun,
    // context that maps to fs::File instance, used with logging callback
    _logging_context: LoggingContext,
    #[cfg(target_os = "android")]
    tun_provider: Arc<Mutex<TunProvider>>,
}

impl WgGoTunnel {
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<Self> {
        #[cfg(target_os = "android")]
        let tun_provider_clone = tun_provider.clone();

        #[cfg_attr(not(target_os = "android"), allow(unused_mut))]
        let (mut tunnel_device, tunnel_fd) = Self::get_tunnel(tun_provider, config, routes)?;

        let interface_name: String = tunnel_device.interface_name().to_string();
        let wg_config_str = config.to_userspace_format();
        let logging_context = initialize_logging(log_path)
            .map(LoggingContext)
            .map_err(TunnelError::LoggingError)?;

        #[cfg(not(target_os = "android"))]
        let mtu = config.mtu as isize;
        let handle = unsafe {
            wgTurnOn(
                #[cfg(not(target_os = "android"))]
                mtu,
                wg_config_str.as_ptr() as _,
                tunnel_fd,
                Some(logging::wg_go_logging_callback),
                logging_context.0 as *mut c_void,
            )
        };
        check_wg_status(handle)?;

        #[cfg(target_os = "android")]
        Self::bypass_tunnel_sockets(&mut tunnel_device, handle)
            .map_err(TunnelError::BypassError)?;

        Ok(WgGoTunnel {
            interface_name,
            handle: Some(handle),
            _tunnel_device: tunnel_device,
            _logging_context: logging_context,
            #[cfg(target_os = "android")]
            tun_provider: tun_provider_clone,
        })
    }

    fn create_tunnel_config(config: &Config, routes: impl Iterator<Item = IpNetwork>) -> TunConfig {
        let mut dns_servers = vec![IpAddr::V4(config.ipv4_gateway)];
        dns_servers.extend(config.ipv6_gateway.map(IpAddr::V6));

        TunConfig {
            addresses: config.tunnel.addresses.clone(),
            dns_servers,
            routes: routes.collect(),
            #[cfg(target_os = "android")]
            required_routes: Self::create_required_routes(config),
            mtu: config.mtu,
        }
    }

    #[cfg(target_os = "android")]
    fn create_required_routes(config: &Config) -> Vec<IpNetwork> {
        let mut required_routes = vec![IpNetwork::new(IpAddr::V4(config.ipv4_gateway), 32)
            .expect("Invalid IPv4 network prefix")];

        required_routes.extend(config.ipv6_gateway.map(|address| {
            IpNetwork::new(IpAddr::V6(address), 128).expect("Invalid IPv6 network prefix")
        }));

        required_routes
    }

    #[cfg(target_os = "android")]
    fn bypass_tunnel_sockets(
        tunnel_device: &mut Tun,
        handle: i32,
    ) -> std::result::Result<(), tun_provider::Error> {
        let socket_v4 = unsafe { wgGetSocketV4(handle) };
        let socket_v6 = unsafe { wgGetSocketV6(handle) };

        tunnel_device.bypass(socket_v4)?;
        tunnel_device.bypass(socket_v6)?;

        Ok(())
    }

    fn stop_tunnel(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            let status = unsafe { wgTurnOff(handle) };
            if status < 0 {
                return Err(TunnelError::StopWireguardError { status });
            }
        }
        Ok(())
    }

    fn get_tunnel(
        tun_provider: Arc<Mutex<TunProvider>>,
        config: &Config,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<(Tun, RawFd)> {
        let mut last_error = None;
        let tunnel_config = Self::create_tunnel_config(config, routes);

        let mut tun_provider = tun_provider.lock().unwrap();

        for _ in 1..=MAX_PREPARE_TUN_ATTEMPTS {
            let tunnel_device = tun_provider
                .get_tun(tunnel_config.clone())
                .map_err(TunnelError::SetupTunnelDevice)?;

            match nix::unistd::dup(tunnel_device.as_raw_fd()) {
                Ok(fd) => return Ok((tunnel_device, fd)),
                #[cfg(not(target_os = "macos"))]
                Err(error @ nix::errno::Errno::EBADFD) => last_error = Some(error),
                Err(error @ nix::errno::Errno::EBADF) => last_error = Some(error),
                Err(error) => return Err(TunnelError::FdDuplicationError(error)),
            }
        }

        Err(TunnelError::FdDuplicationError(
            last_error.expect("Should be collected in loop"),
        ))
    }
}

impl Drop for WgGoTunnel {
    fn drop(&mut self) {
        if let Err(e) = self.stop_tunnel() {
            log::error!("Failed to stop tunnel: {}", e);
        }
    }
}

impl Tunnel for WgGoTunnel {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn get_tunnel_stats(&self) -> Result<StatsMap> {
        let config_str = unsafe {
            let ptr = wgGetConfig(self.handle.unwrap());
            if ptr.is_null() {
                log::error!("Failed to get config !");
                return Err(TunnelError::GetConfigError);
            }

            CStr::from_ptr(ptr)
        };

        let result =
            Stats::parse_config_str(config_str.to_str().expect("Go strings are always UTF-8"))
                .map_err(|error| TunnelError::StatsError(BoxedError::new(error)));
        unsafe {
            // Zeroing out config string to not leave private key in memory.
            let slice = std::slice::from_raw_parts_mut(
                config_str.as_ptr() as *mut c_char,
                config_str.to_bytes().len(),
            );
            slice.zeroize();

            wgFreePtr(config_str.as_ptr() as *mut c_void);
        }

        result
    }

    fn stop(mut self: Box<Self>) -> Result<()> {
        self.stop_tunnel()
    }

    fn set_config(
        &self,
        config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), super::TunnelError>> + Send>> {
        let wg_config_str = config.to_userspace_format();
        let handle = self.handle.unwrap();
        #[cfg(target_os = "android")]
        let tun_provider = self.tun_provider.clone();
        Box::pin(async move {
            let status = unsafe { wgSetConfig(handle, wg_config_str.as_ptr() as _) };
            if status != 0 {
                return Err(TunnelError::SetConfigError);
            }

            // When reapplying the config, the endpoint socket may be discarded
            // and needs to be excluded again
            #[cfg(target_os = "android")]
            {
                let socket_v4 = unsafe { wgGetSocketV4(handle) };
                let socket_v6 = unsafe { wgGetSocketV6(handle) };
                let mut provider = tun_provider.lock().unwrap();
                provider
                    .bypass(socket_v4)
                    .map_err(super::TunnelError::BypassError)?;
                provider
                    .bypass(socket_v6)
                    .map_err(super::TunnelError::BypassError)?;
            }

            Ok(())
        })
    }
}

fn check_wg_status(wg_code: i32) -> Result<()> {
    match wg_code {
        ERROR_GENERAL_FAILURE => Err(TunnelError::FatalStartWireguardError),
        ERROR_INTERMITTENT_FAILURE => Err(TunnelError::RecoverableStartWireguardError),
        0.. => Ok(()),
        _ => {
            log::error!("Unknown status code returned from wireguard-go");
            Err(TunnelError::FatalStartWireguardError)
        }
    }
}

pub type Fd = std::os::unix::io::RawFd;

const ERROR_GENERAL_FAILURE: i32 = -1;
const ERROR_INTERMITTENT_FAILURE: i32 = -2;

extern "C" {
    /// Creates a new wireguard tunnel, uses the specific interface name, MTU and file descriptors
    /// for the tunnel device and logging.
    ///
    /// Positive return values are tunnel handles for this specific wireguard tunnel instance.
    /// Negative return values signify errors. All error codes are opaque.
    #[cfg(not(target_os = "android"))]
    fn wgTurnOn(
        mtu: isize,
        settings: *const i8,
        fd: Fd,
        logging_callback: Option<logging::LoggingCallback>,
        logging_context: *mut c_void,
    ) -> i32;

    // Android
    #[cfg(target_os = "android")]
    fn wgTurnOn(
        settings: *const i8,
        fd: Fd,
        logging_callback: Option<logging::LoggingCallback>,
        logging_context: *mut c_void,
    ) -> i32;

    // Pass a handle that was created by wgTurnOn to stop a wireguard tunnel.
    fn wgTurnOff(handle: i32) -> i32;

    // Returns the file descriptor of the tunnel IPv4 socket.
    fn wgGetConfig(handle: i32) -> *mut c_char;

    // Sets the config of the WireGuard interface.
    fn wgSetConfig(handle: i32, settings: *const i8) -> i32;

    // Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    fn wgFreePtr(ptr: *mut c_void);

    // Returns the file descriptor of the tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    fn wgGetSocketV4(handle: i32) -> Fd;

    // Returns the file descriptor of the tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    fn wgGetSocketV6(handle: i32) -> Fd;
}

mod stats {
    use super::{Stats, StatsMap};

    #[derive(err_derive::Error, Debug, PartialEq)]
    pub enum Error {
        #[error(display = "Failed to parse peer pubkey from string \"{}\"", _0)]
        PubKeyParse(String, #[error(source)] hex::FromHexError),

        #[error(display = "Failed to parse integer from string \"{}\"", _0)]
        IntParse(String, #[error(source)] std::num::ParseIntError),
    }

    impl Stats {
        pub fn parse_config_str(config: &str) -> std::result::Result<StatsMap, Error> {
            let mut map = StatsMap::new();

            let mut peer = None;
            let mut tx_bytes = None;
            let mut rx_bytes = None;

            // parts iterates over keys and values
            let parts = config.split('\n').filter_map(|line| {
                let mut pair = line.split('=');
                let key = pair.next()?;
                let value = pair.next()?;
                Some((key, value))
            });

            for (key, value) in parts {
                match key {
                    "public_key" => {
                        let mut buffer = [0u8; 32];
                        hex::decode_to_slice(value, &mut buffer)
                            .map_err(|err| Error::PubKeyParse(value.to_string(), err))?;
                        peer = Some(buffer);
                        tx_bytes = None;
                        rx_bytes = None;
                    }
                    "rx_bytes" => {
                        rx_bytes = Some(
                            value
                                .trim()
                                .parse()
                                .map_err(|err| Error::IntParse(value.to_string(), err))?,
                        );
                    }
                    "tx_bytes" => {
                        tx_bytes = Some(
                            value
                                .trim()
                                .parse()
                                .map_err(|err| Error::IntParse(value.to_string(), err))?,
                        );
                    }

                    _ => continue,
                }

                if let (Some(peer_val), Some(tx_bytes_val), Some(rx_bytes_val)) =
                    (peer, tx_bytes, rx_bytes)
                {
                    map.insert(
                        peer_val,
                        Self {
                            tx_bytes: tx_bytes_val,
                            rx_bytes: rx_bytes_val,
                        },
                    );
                    peer = None;
                    tx_bytes = None;
                    rx_bytes = None;
                }
            }
            Ok(map)
        }
    }

    #[cfg(test)]
    mod test {
        use super::super::stats::{Error, Stats};

        #[test]
        fn test_parsing() {
            let valid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=2740\nrx_bytes=2396\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
            let pubkey = [0u8; 32];

            let stats = Stats::parse_config_str(valid_input).expect("Failed to parse valid input");
            assert_eq!(stats.len(), 1);
            let actual_keys: Vec<[u8; 32]> = stats.keys().cloned().collect();
            assert_eq!(actual_keys, [pubkey]);
            assert_eq!(stats[&pubkey].rx_bytes, 2396);
            assert_eq!(stats[&pubkey].tx_bytes, 2740);
        }

        #[test]
        fn test_parsing_invalid_input() {
            let invalid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=27error40\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
            let invalid_str = "27error40".to_string();
            let int_err = invalid_str.parse::<u64>().unwrap_err();

            assert_eq!(
                Stats::parse_config_str(invalid_input),
                Err(Error::IntParse(invalid_str, int_err))
            );
        }
    }
}

mod logging {
    use super::super::logging::{log, LogLevel};
    use std::ffi::{c_char, c_void};

    // Callback that receives messages from WireGuard
    pub unsafe extern "system" fn wg_go_logging_callback(
        level: WgLogLevel,
        msg: *const c_char,
        context: *mut c_void,
    ) {
        let managed_msg = if !msg.is_null() {
            std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string()
        } else {
            "Logging message from WireGuard is NULL".to_string()
        };

        let level = match level {
            WG_GO_LOG_VERBOSE => LogLevel::Verbose,
            _ => LogLevel::Error,
        };

        log(context as u32, level, "wireguard-go", &managed_msg);
    }

    // wireguard-go supports log levels 0 through 3 with 3 being the most verbose
    // const WG_GO_LOG_SILENT: WgLogLevel = 0;
    // const WG_GO_LOG_ERROR: WgLogLevel = 1;
    const WG_GO_LOG_VERBOSE: WgLogLevel = 2;

    pub type WgLogLevel = u32;

    pub type LoggingCallback =
        unsafe extern "system" fn(level: WgLogLevel, msg: *const c_char, context: *mut c_void);
}
