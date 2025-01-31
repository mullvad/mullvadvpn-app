use std::{
    future::Future,
    path::Path,
    sync::{Arc, Mutex},
};

    use crate::{
    config::Config,
    stats::{Stats, StatsMap},
    Tunnel, TunnelError,
};
use boringtun::device::{
    api::{command::*, ConfigRx, ConfigTx},
    peer::AllowedIP,
    DeviceConfig, DeviceHandle,
};
use ipnetwork::IpNetwork;
use talpid_tunnel::tun_provider::Tun;
use talpid_tunnel::tun_provider::TunProvider;

#[cfg(unix)]
const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

pub struct BoringTun {
    device_handle: DeviceHandle,
    config_tx: ConfigTx,
    config: Config,
    
    interface_name: String,

    // /// holding on to the tunnel device and the log file ensures that the associated file handles
    // /// live long enough and get closed when the tunnel is stopped
    // tunnel_device: Tun,
}

impl BoringTun {
    pub async fn start_tunnel(
        config: &Config,
        _log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
        #[cfg(daita)] _resource_dir: &Path,
    ) -> Result<Self, TunnelError> {
        log::info!("BoringTun::start_tunnel");

        log::info!("calling get_tunnel_for_userspace");
        // TODO: investigate timing bug when creating tun device? (Device or resource busy)
        let tun = get_tunnel_for_userspace(tun_provider, config, routes)?;

        let interface_name = tun.interface_name();
        let async_tun = tun.into_tun_lol();

        let (mut config_tx, config_rx) = ConfigRx::new();

        let boringtun_config = DeviceConfig {
            n_threads: 4,
            //use_connected_socket: false, // TODO: what is this?
            #[cfg(target_os = "linux")]
            use_multi_queue: false,      // TODO: what is this?
            api: Some(config_rx),
        };

        log::info!("passing tunnel dev to boringtun");
        let device_handle: DeviceHandle =
            // TODO: don't pass file descriptor as a string -_-
            DeviceHandle::new(async_tun, boringtun_config)
                .await
                .map_err(TunnelError::BoringTunDevice)?;

        set_boringtun_config(&mut config_tx, config).await;

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
            interface_name,
            //tunnel_device: tun,
        })
    }
}

impl Tunnel for BoringTun {
    fn get_interface_name(&self) -> String {
        //self.tunnel_device.interface_name()
        self.interface_name.clone()
    }

    fn stop(self: Box<Self>) -> Result<(), TunnelError> {
        // TODO: ASYNC!
        log::info!("BoringTun::stop");
        tokio::spawn(async {
            self.device_handle.stop().await;
        });
        std::thread::sleep_ms(1000);
        Ok(())
    }

    fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        let response = self
            .config_tx
            // TODO: async?
            .send_sync(Get::default())
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

        // TODO:
        let mut tx = self.config_tx.clone();
        Box::pin(async move {
            set_boringtun_config(&mut tx, &config).await;
            Ok(())
        })
    }

    fn start_daita(&mut self) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}

async fn set_boringtun_config(tx: &mut ConfigTx, config: &Config) {
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
                    .map(|net| AllowedIP {
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

    tx.send(set_cmd).await.expect("Failed to configure boringtun");
}

#[cfg(target_os = "windows")]
fn get_tunnel_for_userspace(
    tun_provider: Arc<Mutex<TunProvider>>,
    config: &Config,
    routes: impl Iterator<Item = IpNetwork>,
) -> Result<Tun, crate::TunnelError> {
    let mut tun_provider = tun_provider.lock().unwrap();

    let tun_config = tun_provider.config_mut();
    tun_config.addresses = config.tunnel.addresses.clone();
    tun_config.ipv4_gateway = config.ipv4_gateway;
    tun_config.ipv6_gateway = config.ipv6_gateway;
    tun_config.mtu = config.mtu;

    let _ = routes;

    #[cfg(windows)]
    tun_provider
        .open_tun()
        .map_err(TunnelError::SetupTunnelDevice2)
}

#[cfg(unix)]
fn get_tunnel_for_userspace(
    tun_provider: Arc<Mutex<TunProvider>>,
    config: &Config,
    routes: impl Iterator<Item = IpNetwork>,
) -> Result<Tun, crate::TunnelError> {
    let mut last_error = None;
    let mut tun_provider = tun_provider.lock().unwrap();

    let tun_config = tun_provider.config_mut();
    #[cfg(target_os = "linux")]
    {
        tun_config.name = Some(MULLVAD_INTERFACE_NAME.to_string());
    }
    tun_config.addresses = config.tunnel.addresses.clone();
    tun_config.ipv4_gateway = config.ipv4_gateway;
    tun_config.ipv6_gateway = config.ipv6_gateway;
    tun_config.routes = routes.collect();
    tun_config.mtu = config.mtu;

    for _ in 1..=MAX_PREPARE_TUN_ATTEMPTS {
        let tunnel_device = tun_provider
            .open_tun()
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
