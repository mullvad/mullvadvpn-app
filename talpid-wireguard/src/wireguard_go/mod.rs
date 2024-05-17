use ipnetwork::IpNetwork;
#[cfg(daita)]
use once_cell::sync::OnceCell;
#[cfg(daita)]
use std::{ffi::CString, fs, path::PathBuf};
use std::{
    future::Future,
    net::IpAddr,
    os::unix::io::{AsRawFd, RawFd},
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex},
};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::Error as TunProviderError;
use talpid_tunnel::tun_provider::{Tun, TunConfig, TunProvider};
use talpid_types::BoxedError;

use super::{
    stats::{Stats, StatsMap},
    Config, Tunnel, TunnelError,
};
use crate::logging::{clean_up_logging, initialize_logging};

const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

/// Maximum number of events that can be stored in the underlying buffer
#[cfg(daita)]
const DAITA_EVENTS_CAPACITY: u32 = 1000;

/// Maximum number of actions that can be stored in the underlying buffer
#[cfg(daita)]
const DAITA_ACTIONS_CAPACITY: u32 = 1000;

type Result<T> = std::result::Result<T, TunnelError>;

struct LoggingContext(u64);

impl Drop for LoggingContext {
    fn drop(&mut self) {
        clean_up_logging(self.0);
    }
}

pub struct WgGoTunnel {
    interface_name: String,
    tunnel_handle: wireguard_go_rs::Tunnel,
    // holding on to the tunnel device and the log file ensures that the associated file handles
    // live long enough and get closed when the tunnel is stopped
    _tunnel_device: Tun,
    // context that maps to fs::File instance, used with logging callback
    _logging_context: LoggingContext,
    #[cfg(target_os = "android")]
    tun_provider: Arc<Mutex<TunProvider>>,
    #[cfg(daita)]
    resource_dir: PathBuf,
    #[cfg(daita)]
    config: Config,
}

impl WgGoTunnel {
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
        #[cfg(daita)] resource_dir: &Path,
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
        let handle = wireguard_go_rs::Tunnel::turn_on(
            #[cfg(not(target_os = "android"))]
            mtu,
            &wg_config_str,
            tunnel_fd,
            Some(logging::wg_go_logging_callback),
            logging_context.0,
        )
        .map_err(|e| TunnelError::FatalStartWireguardError(Box::new(e)))?;

        #[cfg(target_os = "android")]
        Self::bypass_tunnel_sockets(&handle, &mut tunnel_device)
            .map_err(TunnelError::BypassError)?;

        Ok(WgGoTunnel {
            interface_name,
            tunnel_handle: handle,
            _tunnel_device: tunnel_device,
            _logging_context: logging_context,
            #[cfg(target_os = "android")]
            tun_provider: tun_provider_clone,
            #[cfg(daita)]
            resource_dir: resource_dir.to_owned(),
            #[cfg(daita)]
            config: config.clone(),
        })
    }

    fn create_tunnel_config(
        config: &Config,
        routes: impl Iterator<Item = IpNetwork>,
        #[cfg(target_os = "android")] excluded_apps: Vec<String>,
    ) -> TunConfig {
        let mut dns_servers = vec![IpAddr::V4(config.ipv4_gateway)];
        dns_servers.extend(config.ipv6_gateway.map(IpAddr::V6));

        TunConfig {
            addresses: config.tunnel.addresses.clone(),
            dns_servers,
            routes: routes.collect(),
            #[cfg(target_os = "android")]
            required_routes: Self::create_required_routes(config),
            #[cfg(target_os = "android")]
            excluded_packages: excluded_apps,
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
        handle: &wireguard_go_rs::Tunnel,
        tunnel_device: &mut Tun,
    ) -> std::result::Result<(), TunProviderError> {
        let socket_v4 = handle.get_socket_v4();
        let socket_v6 = handle.get_socket_v6();

        tunnel_device.bypass(socket_v4)?;
        tunnel_device.bypass(socket_v6)?;

        Ok(())
    }

    fn get_tunnel(
        tun_provider: Arc<Mutex<TunProvider>>,
        config: &Config,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<(Tun, RawFd)> {
        let mut last_error = None;
        let mut tun_provider = tun_provider.lock().unwrap();

        let tunnel_config = Self::create_tunnel_config(
            config,
            routes,
            #[cfg(target_os = "android")]
            tun_provider.get_excluded_apps().collect(),
        );

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

impl Tunnel for WgGoTunnel {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn get_tunnel_stats(&self) -> Result<StatsMap> {
        self.tunnel_handle
            .get_config(|cstr| {
                Stats::parse_config_str(cstr.to_str().expect("Go strings are always UTF-8"))
            })
            .ok_or(TunnelError::GetConfigError)?
            .map_err(|error| TunnelError::StatsError(BoxedError::new(error)))
    }

    fn stop(self: Box<Self>) -> Result<()> {
        self.tunnel_handle
            .turn_off()
            .map_err(|e| TunnelError::StopWireguardError(Box::new(e)))
    }

    fn set_config(
        &mut self,
        config: Config,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let wg_config_str = config.to_userspace_format();

            self.tunnel_handle
                .set_config(&wg_config_str)
                .map_err(|_| TunnelError::SetConfigError)?;

            #[cfg(target_os = "android")]
            let tun_provider = self.tun_provider.clone();

            // When reapplying the config, the endpoint socket may be discarded
            // and needs to be excluded again
            #[cfg(target_os = "android")]
            {
                let socket_v4 = self.tunnel_handle.get_socket_v4();
                let socket_v6 = self.tunnel_handle.get_socket_v6();
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

    #[cfg(daita)]
    fn start_daita(&mut self) -> Result<()> {
        static MAYBENOT_MACHINES: OnceCell<CString> = OnceCell::new();
        let machines = MAYBENOT_MACHINES.get_or_try_init(|| {
            let path = self.resource_dir.join("maybenot_machines");
            log::debug!("Reading maybenot machines from {}", path.display());

            // TODO: errors
            let machines = fs::read_to_string(path).unwrap();
            let machines = CString::new(machines).unwrap();
            Ok(machines)
        })?;

        log::info!("Initializing DAITA for wireguard device");
        let peer_public_key = &self.config.entry_peer.public_key;
        self.tunnel_handle
            .activate_daita(
                peer_public_key.as_bytes(),
                machines,
                DAITA_EVENTS_CAPACITY,
                DAITA_ACTIONS_CAPACITY,
            )
            .map_err(|e| TunnelError::StartDaita(Box::new(e)))?;

        Ok(())
    }
}

mod stats {
    use super::{Stats, StatsMap};

    #[derive(thiserror::Error, Debug, PartialEq)]
    pub enum Error {
        #[error("Failed to parse peer pubkey from string \"{0}\"")]
        PubKeyParse(String, #[source] hex::FromHexError),

        #[error("Failed to parse integer from string \"{0}\"")]
        IntParse(String, #[source] std::num::ParseIntError),
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
    use std::ffi::c_char;

    // Callback that receives messages from WireGuard
    pub unsafe extern "system" fn wg_go_logging_callback(
        level: WgLogLevel,
        msg: *const c_char,
        context: u64,
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

        log(context, level, "wireguard-go", &managed_msg);
    }

    // wireguard-go supports log levels 0 through 3 with 3 being the most verbose
    // const WG_GO_LOG_SILENT: WgLogLevel = 0;
    // const WG_GO_LOG_ERROR: WgLogLevel = 1;
    const WG_GO_LOG_VERBOSE: WgLogLevel = 2;

    pub type WgLogLevel = u32;
}
