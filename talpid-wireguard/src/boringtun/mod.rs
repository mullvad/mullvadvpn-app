use std::{
    future::Future,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    config::Config,
    stats::{Stats, StatsMap},
    wireguard_go::get_tunnel_for_userspace,
    Tunnel, TunnelError,
};
use boringtun::device::{
    api::{command::*, ConfigRx, ConfigTx},
    peer::AllowedIp,
    DeviceConfig, DeviceHandle,
};
use ipnetwork::IpNetwork;
use talpid_tunnel::tun_provider::Tun;
use talpid_tunnel::tun_provider::TunProvider;

const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

pub struct BoringTun {
    device_handle: DeviceHandle,
    config_tx: ConfigTx,
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

        let (mut config_tx, config_rx) = ConfigRx::new();

        let boringtun_config = DeviceConfig {
            n_threads: 4,
            use_connected_socket: false, // TODO: what is this?
            use_multi_queue: false,      // TODO: what is this?
            api: Some(config_rx),
        };

        log::info!("passing tunnel dev to boringtun");
        let device_handle: DeviceHandle =
            // TODO: don't pass file descriptor as a string -_-
            DeviceHandle::new(&_tunnel_fd.to_string(), boringtun_config)
                .map_err(TunnelError::BoringTunDevice)?;

        set_boringtun_config(&mut config_tx, config);

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
            config_tx,
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
        let response = self
            .config_tx
            .send(Get::default())
            .expect("Failed to get peers");

        let Response::Get(response) = response else {
            panic!();
        };

        let mut stats = StatsMap::new();

        for peer in &response.peers {
            stats.insert(
                peer.peer.public_key.0,
                Stats {
                    tx_bytes: peer.tx_bytes.unwrap_or_default(),
                    rx_bytes: peer.rx_bytes.unwrap_or_default(),
                },
            );
        }
        Ok(stats)
    }

    fn set_config<'a>(
        &'a mut self,
        config: Config,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TunnelError>> + Send + 'a>> {
        self.config = config.clone();
        set_boringtun_config(&mut self.config_tx, &config);

        // TODO:
        Box::pin(async { Ok(()) })
    }

    fn start_daita(&mut self) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}

fn set_boringtun_config(tx: &mut ConfigTx, config: &Config) {
    log::info!("configuring boringtun device");
    let mut set_cmd = Set::builder()
        .private_key(config.tunnel.private_key.to_bytes())
        .listen_port(0u16)
        .replace_peers()
        .build();

    #[cfg(target_os = "linux")]
    {
        set_cmd.fwmark = config.fwmark;
    }

    for peer in config.peers() {
        let mut boring_peer = Peer::builder()
            .public_key(*peer.public_key.as_bytes())
            .endpoint(peer.endpoint)
            .allowed_ip(
                peer.allowed_ips
                    .iter()
                    .map(|net| AllowedIp {
                        addr: net.ip(),
                        cidr: net.prefix(),
                    })
                    .collect(),
            )
            .build();

        if let Some(psk) = &peer.psk {
            boring_peer.preshared_key = Some(SetUnset::Set((*psk.as_bytes()).into()));
        }

        let boring_peer = SetPeer::builder().peer(boring_peer).build();

        set_cmd.peers.push(boring_peer);
    }

    tx.send(set_cmd).expect("Failed to configure boringtun");
}
