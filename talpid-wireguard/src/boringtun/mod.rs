use std::{
    future::Future,
    os::fd::{AsRawFd, RawFd},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    config::{Config, MULLVAD_INTERFACE_NAME},
    stats::{Stats, StatsMap},
    wireguard_go::get_tunnel_for_userspace,
    Tunnel, TunnelError,
};
use boringtun::device::{DeviceConfig, DeviceHandle};
use ipnetwork::IpNetwork;
use nix::unistd::{close, write};
use talpid_tunnel::tun_provider::Tun;
use talpid_tunnel::tun_provider::TunProvider;
use talpid_types::net::wireguard::PeerConfig;

const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

pub struct BoringTun {
    device_handle: DeviceHandle,
    config: Config,

    /// holding on to the tunnel device and the log file ensures that the associated file handles
    /// live long enough and get closed when the tunnel is stopped
    tunnel_device: Tun,
}

impl BoringTun {
    pub fn start_tunnel(
        config: &Config,
        _log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
        #[cfg(daita)] _resource_dir: &Path,
    ) -> Result<Self, TunnelError> {
        log::info!("BoringTun::start_tunnel");

        log::info!("calling get_tunnel_for_userspace");
        // TODO: investigate timing bug when creating tun device? (Device or resource busy)
        let (tun, _tunnel_fd) = get_tunnel_for_userspace(tun_provider, config, routes)?;

        // TODO: avoid going through a CString
        let config_string = format!(
            "set=1\n{}",
            config.to_userspace_format().to_string_lossy().to_string()
        );
        let boringtun_config = DeviceConfig {
            n_threads: 4,
            use_connected_socket: false, // TODO: what is this?
            use_multi_queue: false,      // TODO: what is this?
            uapi_fd: -1,
            config_string: Some(config_string),
        };

        log::info!("passing tunnel dev to boringtun");
        let device_handle: DeviceHandle =
            // TODO: don't pass file descriptor as a string -_-
            DeviceHandle::new(&_tunnel_fd.to_string(), boringtun_config)
                .map_err(TunnelError::BoringTunDevice)?;

        log::info!(
            "This tunnel was brought to you by...
.........................................................
..*...*.. .--.                    .---.         ..*....*.
...*..... |   )         o           |           ......*..
.*..*..*. |--:  .-. .--..  .--. .-..|.  . .--.  ...*.....
...*..... |   )(   )|   |  |  |(   |||  | |  |  .*.....*.
*.....*.. '--'  `-' ' -' `-'  `-`-`|'`--`-'  `- .....*...
.........                       ._.'            ..*...*..
..*...*.............................................*...."
        );

        Ok(Self {
            device_handle,
            config: config.clone(),
            tunnel_device: tun,
        })
    }
}

impl Tunnel for BoringTun {
    fn get_interface_name(&self) -> String {
        self.tunnel_device.interface_name().to_string()
    }

    fn stop(mut self: Box<Self>) -> Result<(), TunnelError> {
        log::info!("BoringTun::stop");
        self.device_handle.clean();
        //self.device_handle.wait(); // TODO: do we need this<?

        Ok(())
    }

    fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        let mut stats = StatsMap::new();

        for peer in self.config.peers() {
            stats.insert(
                *peer.public_key.as_bytes(),
                Stats {
                    tx_bytes: 1234,
                    rx_bytes: 4321,
                },
            );
        }
        Ok(stats)
    }

    fn set_config<'a>(
        &'a mut self,
        _config: Config,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TunnelError>> + Send + 'a>> {
        self.config = _config;
        todo!("set_config")
    }

    fn start_daita(&mut self) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}
