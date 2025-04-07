use crate::{
    config::Config,
    stats::{Stats, StatsMap},
    Tunnel, TunnelError,
};
use boringtun::device::{
    api::{command::*, ApiClient, ApiServer},
    peer::AllowedIP,
    DeviceConfig, DeviceHandle,
};
use ipnetwork::IpNetwork;
#[cfg(target_os = "android")]
use std::os::fd::AsRawFd;
use std::{
    future::Future,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex},
};
use talpid_tunnel::tun_provider::{Tun, TunProvider};
use talpid_tunnel_config_client::DaitaSettings;
use tun::AbstractDevice;

pub struct BoringTun {
    device_handle: DeviceHandle,
    config_tx: ApiClient,
    config: Config,

    /// Name of the tun interface.
    interface_name: String,
}

impl BoringTun {
    pub async fn start_tunnel(
        config: &Config,
        _log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
        #[cfg(target_os = "android")] route_manager: talpid_routing::RouteManagerHandle,
    ) -> Result<Self, TunnelError> {
        log::info!("BoringTun::start_tunnel");

        log::info!("calling get_tunnel_for_userspace");
        // TODO: investigate timing bug when creating tun device? (Device or resource busy)
        #[cfg(not(target_os = "android"))]
        let async_tun = {
            let tun = crate::boringtun::get_tunnel_for_userspace(tun_provider, config, routes)?;

            #[cfg(unix)]
            {
                tun.into_inner().into_inner()
            }
            #[cfg(windows)]
            {
                tun.into_tun_lol()
            }
        };

        let (mut config_tx, config_rx) = ApiServer::new();

        let boringtun_config = DeviceConfig {
            n_threads: 4,
            #[cfg(target_os = "linux")]
            use_multi_queue: false, // TODO: what is this?
            api: Some(config_rx),
            on_bind: None,
        };

        #[cfg(target_os = "android")]
        let mut boringtun_config = boringtun_config;

        #[cfg(target_os = "android")]
        let async_tun = {
            let _ = routes; // TODO: do we need this?
            let (mut tun, fd) = get_tunnel_for_userspace(Arc::clone(&tun_provider), config)?;
            let is_new_tunnel = tun.is_new;

            // TODO We should also wait for routes before sending any ping / connectivity check

            // There is a brief period of time between setting up a Wireguard-go tunnel and the tunnel being ready to serve
            // traffic. This function blocks until the tunnel starts to serve traffic or until [connectivity::Check] times out.
            if is_new_tunnel {
                let expected_routes = tun_provider.lock().unwrap().real_routes();

                route_manager
                    .clone()
                    .wait_for_routes(expected_routes)
                    .await
                    .map_err(crate::Error::SetupRoutingError)
                    .map_err(|e| TunnelError::RecoverableStartWireguardError(Box::new(e)))?;
            }

            let mut config = tun::Configuration::default();
            config.raw_fd(fd);

            boringtun_config.on_bind = Some(Box::new(move |socket| {
                tun.bypass(socket.as_raw_fd()).unwrap()
            }));

            let device = tun::Device::new(&config).unwrap();
            tun::AsyncDevice::new(device).unwrap()
        };

        let interface_name = async_tun.deref().tun_name().unwrap();

        log::info!("passing tunnel dev to boringtun");
        let device_handle: DeviceHandle = DeviceHandle::new(async_tun, boringtun_config)
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
        })
    }
}

#[async_trait::async_trait]
impl Tunnel for BoringTun {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn stop(self: Box<Self>) -> Result<(), TunnelError> {
        log::info!("BoringTun::stop"); // remove me
        tokio::runtime::Handle::current().block_on(self.device_handle.stop());
        Ok(())
    }

    async fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        let response = self
            .config_tx
            .send(Get::default())
            .await
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
        Box::pin(async move {
            self.config = config;
            set_boringtun_config(&mut self.config_tx, &self.config).await;
            Ok(())
        })
    }

    fn start_daita(&mut self, _settings: DaitaSettings) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}

async fn set_boringtun_config(tx: &mut ApiClient, config: &Config) {
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

    tx.send(set_cmd)
        .await
        .expect("Failed to configure boringtun");
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

#[cfg(all(not(target_os = "android"), unix))]
fn get_tunnel_for_userspace(
    tun_provider: Arc<Mutex<TunProvider>>,
    config: &Config,
    routes: impl Iterator<Item = IpNetwork>,
) -> Result<Tun, crate::TunnelError> {
    let mut tun_provider = tun_provider.lock().unwrap();

    let tun_config = tun_provider.config_mut();
    #[cfg(target_os = "linux")]
    {
        tun_config.name = Some(crate::config::MULLVAD_INTERFACE_NAME.to_string());
        tun_config.packet_information = false;
    }
    tun_config.addresses = config.tunnel.addresses.clone();
    tun_config.ipv4_gateway = config.ipv4_gateway;
    tun_config.ipv6_gateway = config.ipv6_gateway;
    tun_config.routes = routes.collect();
    tun_config.mtu = config.mtu;

    tun_provider
        .open_tun()
        .map_err(TunnelError::SetupTunnelDevice)
}

#[cfg(target_os = "android")]
pub fn get_tunnel_for_userspace(
    tun_provider: Arc<Mutex<TunProvider>>,
    config: &Config,
) -> Result<(Tun, std::os::fd::RawFd), TunnelError> {
    let mut last_error = None;
    let mut tun_provider = tun_provider.lock().unwrap();

    let tun_config = tun_provider.config_mut();
    tun_config.addresses = config.tunnel.addresses.clone();
    tun_config.ipv4_gateway = config.ipv4_gateway;
    tun_config.ipv6_gateway = config.ipv6_gateway;
    tun_config.mtu = config.mtu;

    // Route everything into the tunnel and have wireguard-go act as a firewall when
    // blocking. These will not necessarily be the actual routes used by android. Those will
    // be generated at a later stage e.g. if Local Network Sharing is enabled.
    tun_config.routes = vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()];

    const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

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
