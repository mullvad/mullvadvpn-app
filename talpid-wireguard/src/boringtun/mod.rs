use crate::{
    config::Config,
    stats::{Stats, StatsMap},
    Tunnel, TunnelError,
};
#[cfg(target_os = "android")]
use boringtun::udp::UdpTransportFactory;
use boringtun::{
    device::{
        api::{command::*, ApiClient, ApiServer},
        peer::AllowedIP,
        DeviceConfig, DeviceHandle,
    },
    udp::{channel::PacketChannel, UdpSocketFactory},
};
#[cfg(not(target_os = "android"))]
use ipnetwork::IpNetwork;
#[cfg(target_os = "android")]
use std::os::fd::IntoRawFd;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::Deref,
    sync::{Arc, Mutex},
};
use talpid_tunnel::tun_provider::{self, Tun, TunProvider};
use talpid_tunnel_config_client::DaitaSettings;
use tun07::{AbstractDevice, AsyncDevice};

#[cfg(target_os = "android")]
type UdpFactory = AndroidUdpSocketFactory;

#[cfg(not(target_os = "android"))]
type UdpFactory = UdpSocketFactory;

type SinglehopDevice = DeviceHandle<(UdpFactory, Arc<tun07::AsyncDevice>, Arc<tun07::AsyncDevice>)>;
type EntryDevice = DeviceHandle<(UdpFactory, PacketChannel, PacketChannel)>;
type ExitDevice = DeviceHandle<(PacketChannel, Arc<AsyncDevice>, Arc<AsyncDevice>)>;

pub struct BoringTun {
    /// Device handles
    devices: Devices,

    /// Tunnel config
    config: Config,

    /// Name of the tun interface.
    interface_name: String,
}

enum Devices {
    Singlehop {
        device: SinglehopDevice,
        api: ApiClient,
    },

    Multihop {
        entry_device: EntryDevice,
        entry_api: ApiClient,

        exit_device: ExitDevice,
        exit_api: ApiClient,
    },
}

#[cfg(target_os = "android")]
struct AndroidUdpSocketFactory {
    pub tun: Tun,
}

#[cfg(target_os = "android")]
impl UdpTransportFactory for AndroidUdpSocketFactory {
    type Transport = <UdpSocketFactory as UdpTransportFactory>::Transport;

    async fn bind(
        &mut self,
        params: &boringtun::udp::UdpTransportFactoryParams,
    ) -> std::io::Result<(Arc<Self::Transport>, Arc<Self::Transport>)> {
        let (udp_v4, udp_v6) = UdpSocketFactory.bind(params).await?;

        self.tun.bypass(&udp_v4).unwrap();
        self.tun.bypass(&udp_v6).unwrap();

        Ok((udp_v4, udp_v6))
    }
}

/// Configure and start a boringtun tunnel.
pub async fn open_boringtun_tunnel(
    config: &Config,
    tun_provider: Arc<Mutex<tun_provider::TunProvider>>,
    #[cfg(target_os = "android")] route_manager_handle: talpid_routing::RouteManagerHandle,
) -> super::Result<BoringTun> {
    log::info!("BoringTun::start_tunnel");
    let routes = config.get_tunnel_destinations();

    log::info!("calling get_tunnel_for_userspace");
    #[cfg(not(target_os = "android"))]
    let async_tun = {
        let tun = get_tunnel_for_userspace(tun_provider, config, routes)?;

        #[cfg(unix)]
        {
            tun.into_inner().into_inner()
        }
        #[cfg(windows)]
        {
            tun.into_inner()
        }
    };

    let (entry_api, entry_api_server) = ApiServer::new();
    let boringtun_entry_config = DeviceConfig {
        n_threads: 4,
        api: Some(entry_api_server),
    };

    #[cfg(target_os = "android")]
    let (tun, async_tun) = {
        let _ = routes; // TODO: do we need this?
        let (tun, fd) = get_tunnel_for_userspace(Arc::clone(&tun_provider), config)?;
        let is_new_tunnel = tun.is_new;

        // TODO We should also wait for routes before sending any ping / connectivity check

        // There is a brief period of time between setting up a Wireguard-go tunnel and the tunnel being ready to serve
        // traffic. This function blocks until the tunnel starts to serve traffic or until [connectivity::Check] times out.
        if is_new_tunnel {
            let expected_routes = tun_provider.lock().unwrap().real_routes();

            route_manager_handle
                .clone()
                .wait_for_routes(expected_routes)
                .await
                .map_err(crate::Error::SetupRoutingError)
                .map_err(|e| TunnelError::RecoverableStartWireguardError(Box::new(e)))?;
        }

        let mut tun_config = tun07::Configuration::default();
        tun_config.raw_fd(fd);

        let device = tun07::Device::new(&tun_config).unwrap();

        (tun, tun07::AsyncDevice::new(device).unwrap())
    };

    let interface_name = async_tun.deref().tun_name().unwrap();

    log::info!("passing tunnel dev to boringtun");
    let async_tun = Arc::new(async_tun);

    let mut boringtun = if config.exit_peer.is_some() {
        // multihop

        let source_v4 = config.tunnel.addresses.iter().find_map(|ip| match ip {
            &IpAddr::V4(ipv4_addr) => Some(ipv4_addr),
            IpAddr::V6(..) => None,
        });

        let source_v6 = config.tunnel.addresses.iter().find_map(|ip| match ip {
            &IpAddr::V6(ipv6_addr) => Some(ipv6_addr),
            IpAddr::V4(..) => None,
        });

        let channel = PacketChannel::new(
            100,
            source_v4.unwrap_or(Ipv4Addr::UNSPECIFIED), // HACK: unwrap_or
            source_v6.unwrap_or(Ipv6Addr::UNSPECIFIED), // HACK: unwrap_or
        );

        let (exit_api, exit_api_server) = ApiServer::new();
        let exit_device = DeviceHandle::<(PacketChannel, Arc<AsyncDevice>, Arc<AsyncDevice>)>::new(
            channel.clone(),
            async_tun.clone(),
            async_tun,
            DeviceConfig {
                n_threads: 4,
                api: Some(exit_api_server),
            },
        )
        .await
        .map_err(TunnelError::BoringTunDevice)?;

        #[cfg(target_os = "android")]
        let factory = AndroidUdpSocketFactory { tun };

        #[cfg(not(target_os = "android"))]
        let factory = UdpSocketFactory;

        let entry_device =
            EntryDevice::new(factory, channel.clone(), channel, boringtun_entry_config)
                .await
                .map_err(TunnelError::BoringTunDevice)?;

        //set_entry_boringtun_config(&mut entry_api, config).await?;
        //set_exit_boringtun_config(&mut exit_api, config).await?;
        BoringTun {
            config: config.clone(),
            interface_name,
            devices: Devices::Multihop {
                entry_device,
                entry_api,
                exit_device,
                exit_api,
            },
        }
    } else {
        #[cfg(target_os = "android")]
        let factory = AndroidUdpSocketFactory { tun };

        #[cfg(not(target_os = "android"))]
        let factory = UdpSocketFactory;

        let device = SinglehopDevice::new(
            factory,
            async_tun.clone(),
            async_tun,
            boringtun_entry_config,
        )
        .await
        .map_err(TunnelError::BoringTunDevice)?;

        //set_boringtun_config(&mut entry_api, config).await?;

        BoringTun {
            devices: Devices::Singlehop {
                device,
                api: entry_api,
            },
            config: config.clone(),
            interface_name,
        }
    };

    // FIXME: double clone
    boringtun.set_config(config.clone()).await?;

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

    Ok(boringtun)
}

#[async_trait::async_trait]
impl Tunnel for BoringTun {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn stop(self: Box<Self>) -> Result<(), TunnelError> {
        log::info!("BoringTun::stop"); // remove me
        tokio::runtime::Handle::current().block_on(async {
            match self.devices {
                Devices::Singlehop { device, .. } => {
                    device.stop().await;
                }
                Devices::Multihop {
                    entry_device,
                    exit_device,
                    ..
                } => {
                    exit_device.stop().await;
                    entry_device.stop().await;
                }
            }
        });

        Ok(())
    }

    async fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        let mut stats = StatsMap::default();

        let apis;

        match &self.devices {
            Devices::Singlehop { api, .. } => apis = [Some(api), None],
            Devices::Multihop {
                entry_api,
                exit_api,
                ..
            } => apis = [Some(entry_api), Some(exit_api)],
        }

        for api in apis.into_iter().flatten() {
            let response = api.send(Get::default()).await.expect("Failed to get peers");

            let Response::Get(response) = response else {
                return Err(TunnelError::GetConfigError);
            };

            for peer in response.peers {
                stats.insert(
                    peer.peer.public_key.0,
                    Stats {
                        tx_bytes: peer.tx_bytes.unwrap_or_default(),
                        rx_bytes: peer.rx_bytes.unwrap_or_default(),
                    },
                );
            }
        }

        Ok(stats)
    }

    fn set_config<'a>(
        &'a mut self,
        config: Config,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TunnelError>> + Send + 'a>> {
        Box::pin(async move {
            self.config = config;
            match &mut self.devices {
                Devices::Singlehop { api, .. } => {
                    assert!(
                        self.config.exit_peer.is_none(),
                        "todo: support switching between single and multihop"
                    );
                    set_boringtun_config(api, &self.config).await?;
                }
                Devices::Multihop {
                    entry_api,
                    exit_api,
                    ..
                } => {
                    assert!(
                        self.config.exit_peer.is_some(),
                        "todo: support switching between single and multihop"
                    );
                    set_boringtun_entry_config(entry_api, &self.config).await?;
                    set_boringtun_exit_config(exit_api, &self.config).await?;
                }
            }
            Ok(())
        })
    }

    fn start_daita(&mut self, _settings: DaitaSettings) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}

async fn set_boringtun_config(
    tx: &mut ApiClient,
    config: &Config,
) -> Result<(), crate::TunnelError> {
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

    tx.send(set_cmd).await.map_err(|err| {
        log::error!("Failed to set boringtun config: {err:#}");
        TunnelError::SetConfigError
    })?;
    Ok(())
}

async fn set_boringtun_entry_config(
    tx: &mut ApiClient,
    config: &Config,
) -> Result<(), crate::TunnelError> {
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

    let peer = &config.entry_peer;
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

    tx.send(set_cmd).await.map_err(|err| {
        log::error!("Failed to set boringtun config: {err:#}");
        TunnelError::SetConfigError
    })?;
    Ok(())
}

async fn set_boringtun_exit_config(
    tx: &mut ApiClient,
    config: &Config,
) -> Result<(), crate::TunnelError> {
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

    // TODO: don't unwrap
    let peer = config.exit_peer.as_ref().unwrap();

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

    tx.send(set_cmd).await.map_err(|err| {
        log::error!("Failed to set boringtun config: {err:#}");
        TunnelError::SetConfigError
    })?;
    Ok(())
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
        .map_err(TunnelError::SetupTunnelDevice)
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

        match nix::unistd::dup(&tunnel_device) {
            Ok(fd) => return Ok((tunnel_device, fd.into_raw_fd())),
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
