use crate::{
    Tunnel, TunnelError,
    config::Config,
    stats::{Stats, StatsMap},
};
#[cfg(target_os = "android")]
use boringtun::udp::UdpTransportFactory;
use boringtun::{
    device::{
        DeviceConfig, DeviceHandle,
        api::{ApiClient, ApiServer, command::*},
        peer::AllowedIP,
    },
    udp::{
        UdpSocketFactory,
        channel::{PacketChannelUdp, TunChannelRx, TunChannelTx, get_packet_channels},
    },
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
    time::{Duration, SystemTime},
};
use talpid_tunnel::tun_provider::{self, Tun, TunProvider};
use talpid_tunnel_config_client::DaitaSettings;
use tun07::{AbstractDevice, AsyncDevice};
use tunnel_obfuscation::PacketChannelSimple;

#[cfg(target_os = "android")]
type UdpFactory = AndroidUdpSocketFactory;

#[cfg(not(target_os = "android"))]
type UdpFactory = UdpSocketFactory;

type SinglehopDevice = DeviceHandle<(UdpFactory, Arc<tun07::AsyncDevice>, Arc<tun07::AsyncDevice>)>;
type Obfuscator = DeviceHandle<(
    PacketChannelSimple,
    Arc<tun07::AsyncDevice>,
    Arc<tun07::AsyncDevice>,
)>;

type EntryDevice = DeviceHandle<(UdpFactory, TunChannelTx, TunChannelRx)>;
type ExitDevice = DeviceHandle<(PacketChannelUdp, Arc<AsyncDevice>, Arc<AsyncDevice>)>;

// Boringtun -Channel-> Masque-client -UdpSocket-> Network -> Masque-sever (entry) -> WG Relay (exit)

const PACKET_CHANNEL_CAPACITY: usize = 100;

pub struct BoringTun {
    /// Device handles
    // TODO: Can we not store this in an option?
    devices: Option<Devices>,

    tun: Arc<AsyncDevice>,

    #[cfg(target_os = "android")]
    android_tun: Arc<Tun>,

    /// Tunnel config
    config: Config,

    /// Name of the tun interface.
    interface_name: String,
}

impl BoringTun {
    async fn new(
        tun: Arc<AsyncDevice>,
        #[cfg(target_os = "android")] android_tun: Arc<Tun>,
        config: Config,
        interface_name: String,
        obfuscator_channels: Option<PacketChannelSimple>,
    ) -> Result<Self, TunnelError> {
        let devices = create_devices(
            &config,
            tun.clone(),
            #[cfg(target_os = "android")]
            android_tun.clone(),
            obfuscator_channels,
        )
        .await?;
        Ok(Self {
            config,
            interface_name,
            tun,
            #[cfg(target_os = "android")]
            android_tun,
            devices: Some(devices),
        })
    }
}

enum Devices {
    // an obfuscated boringtun tunnel will first go through the obfuscator (in-process) before
    // leaving the physical interface.
    Obufscated {
        device: Obfuscator,
        api: ApiClient,
    },
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
    pub tun: Arc<Tun>,
}

#[cfg(target_os = "android")]
impl UdpTransportFactory for AndroidUdpSocketFactory {
    type Send = <UdpSocketFactory as UdpTransportFactory>::Send;
    type RecvV4 = <UdpSocketFactory as UdpTransportFactory>::RecvV4;
    type RecvV6 = <UdpSocketFactory as UdpTransportFactory>::RecvV6;

    async fn bind(
        &mut self,
        params: &boringtun::udp::UdpTransportFactoryParams,
    ) -> std::io::Result<((Self::Send, Self::RecvV4), (Self::Send, Self::RecvV6))> {
        let ((udp_v4_tx, udp_v4_rx), (udp_v6_tx, udp_v6_rx)) =
            UdpSocketFactory.bind(params).await?;

        self.tun.bypass(&udp_v4_tx).unwrap();
        self.tun.bypass(&udp_v6_tx).unwrap();

        Ok(((udp_v4_tx, udp_v4_rx), (udp_v6_tx, udp_v6_rx)))
    }
}

/// Configure and start a boringtun tunnel.
pub async fn open_boringtun_tunnel(
    config: &Config,
    tun_provider: Arc<Mutex<tun_provider::TunProvider>>,
    obfuscator_channels: Option<PacketChannelSimple>,
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

        (Arc::new(tun), tun07::AsyncDevice::new(device).unwrap())
    };

    let interface_name = async_tun.deref().tun_name().unwrap();

    log::info!("passing tunnel dev to boringtun");
    let async_tun = Arc::new(async_tun);

    // TODO: Pass the channel to the obfuscator here v
    // This could def be done via `config`, then we can match on it in `Boringtun::new`.
    let boringtun = BoringTun::new(
        async_tun,
        #[cfg(target_os = "android")]
        tun.clone(),
        config.clone(),
        interface_name,
        obfuscator_channels,
    )
    .await
    .inspect_err(|e| log::error!("Failed to open BoringTun: {e:?}"))?;

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

async fn create_devices(
    config: &Config,
    async_tun: Arc<AsyncDevice>,
    #[cfg(target_os = "android")] tun: Arc<Tun>,
    obfuscator_channels: Option<PacketChannelSimple>,
) -> Result<Devices, TunnelError> {
    let (entry_api, entry_api_server) = ApiServer::new();
    let boringtun_entry_config = DeviceConfig {
        api: Some(entry_api_server),
    };

    if let Some(exit_peer) = &config.exit_peer {
        // multihop

        let source_v4 = config
            .tunnel
            .addresses
            .iter()
            .find_map(|ip| match ip {
                &IpAddr::V4(ipv4_addr) => Some(ipv4_addr),
                IpAddr::V6(..) => None,
            })
            .unwrap_or(Ipv4Addr::UNSPECIFIED);

        let source_v6 = config
            .tunnel
            .addresses
            .iter()
            .find_map(|ip| match ip {
                &IpAddr::V6(ipv6_addr) => Some(ipv6_addr),
                IpAddr::V4(..) => None,
            })
            .unwrap_or(Ipv6Addr::UNSPECIFIED);

        let (tun_tx, tun_rx, udp_channels) =
            get_packet_channels(PACKET_CHANNEL_CAPACITY, source_v4, source_v6);

        let (exit_api, exit_api_server) = ApiServer::new();
        // NOTE: Boringtun will be equiv to ExitDevice here. So this code will look the same, but
        // we need a `get_packet_channels` that returns UdpChannel{Rx,Tx} instead of IpChannel{Rx,Tx}.
        let exit_device = ExitDevice::new(
            udp_channels,
            async_tun.clone(),
            async_tun,
            DeviceConfig {
                api: Some(exit_api_server),
            },
        )
        .await;

        #[cfg(target_os = "android")]
        let factory = AndroidUdpSocketFactory { tun };

        #[cfg(not(target_os = "android"))]
        let factory = UdpSocketFactory;

        let entry_device = EntryDevice::new(factory, tun_tx, tun_rx, boringtun_entry_config).await;

        let private_key = &config.tunnel.private_key;
        let peer = &config.entry_peer;
        let set_cmd = create_set_command(
            #[cfg(target_os = "linux")]
            config.fwmark,
            private_key,
            peer,
        );
        entry_api.send(set_cmd).await.map_err(|err| {
            log::error!("Failed to set boringtun config: {err:#}");
            TunnelError::SetConfigError
        })?;

        let set_cmd = create_set_command(
            #[cfg(target_os = "linux")]
            config.fwmark,
            private_key,
            exit_peer,
        );
        exit_api.send(set_cmd).await.map_err(|err| {
            log::error!("Failed to set boringtun config: {err:#}");
            TunnelError::SetConfigError
        })?;

        Ok(Devices::Multihop {
            entry_device,
            entry_api,
            exit_device,
            exit_api,
        })
    } else {
        match obfuscator_channels {
            Some(yaboi) => {
                let device =
                    Obfuscator::new(yaboi, async_tun.clone(), async_tun, boringtun_entry_config)
                        .await;

                log::info!("configuring boringtun device");
                let private_key = &config.tunnel.private_key;
                let peer = &config.entry_peer;
                let set_cmd = create_set_command(
                    #[cfg(target_os = "linux")]
                    config.fwmark,
                    private_key,
                    peer,
                );

                entry_api.send(set_cmd).await.map_err(|err| {
                    log::error!("Failed to set boringtun config: {err:#}");
                    TunnelError::SetConfigError
                })?;

                Ok(Devices::Obufscated {
                    device,
                    api: entry_api,
                })
            }
            None => {
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
                .await;

                log::info!("configuring boringtun device");
                let private_key = &config.tunnel.private_key;
                let peer = &config.entry_peer;
                let set_cmd = create_set_command(
                    #[cfg(target_os = "linux")]
                    config.fwmark,
                    private_key,
                    peer,
                );

                entry_api.send(set_cmd).await.map_err(|err| {
                    log::error!("Failed to set boringtun config: {err:#}");
                    TunnelError::SetConfigError
                })?;

                Ok(Devices::Singlehop {
                    device,
                    api: entry_api,
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl Tunnel for BoringTun {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn stop(mut self: Box<Self>) -> Result<(), TunnelError> {
        log::info!("BoringTun::stop"); // remove me
        tokio::runtime::Handle::current().block_on(async {
            match self.devices.take().unwrap() {
                Devices::Obufscated { device, .. } => device.stop().await,
                Devices::Singlehop { device, .. } => device.stop().await,
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

        let apis = match self.devices.as_ref().unwrap() {
            Devices::Obufscated { api, .. } => [Some(api), None],
            Devices::Singlehop { api, .. } => [Some(api), None],
            Devices::Multihop {
                entry_api,
                exit_api,
                ..
            } => [Some(entry_api), Some(exit_api)],
        };

        for api in apis.into_iter().flatten() {
            let response = api.send(Get::default()).await.expect("Failed to get peers");

            let Response::Get(response) = response else {
                return Err(TunnelError::GetConfigError);
            };

            for peer in response.peers {
                let last_handshake = || -> Option<SystemTime> {
                    let handshake_sec = peer.last_handshake_time_sec?;
                    let handshake_nsec = peer.last_handshake_time_nsec?;
                    // TODO: Boringtun should probably return a Unix timestamp (like wg-go)
                    Some(SystemTime::now() - Duration::new(handshake_sec, handshake_nsec))
                };

                stats.insert(
                    peer.peer.public_key.0,
                    Stats {
                        tx_bytes: peer.tx_bytes.unwrap_or_default(),
                        rx_bytes: peer.rx_bytes.unwrap_or_default(),
                        last_handshake_time: last_handshake(),
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
            let old_config = std::mem::replace(&mut self.config, config);

            if old_config.is_multihop() != self.config.is_multihop() {
                // TODO: Update existing tunnels?
                match self.devices.take().unwrap() {
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
                    Devices::Obufscated { device, api } => device.stop().await,
                }

                self.devices = Some(
                    create_devices(
                        &self.config,
                        self.tun.clone(),
                        #[cfg(target_os = "android")]
                        self.android_tun.clone(),
                        None, // TODO: Pass obfuscator_channels if `self` is of variant Obfuscated.
                    )
                    .await?,
                );
            }
            Ok(())
        })
    }

    fn start_daita(&mut self, _settings: DaitaSettings) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}

fn create_set_command(
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
    private_key: &talpid_types::net::wireguard::PrivateKey,
    peer: &talpid_types::net::wireguard::PeerConfig,
) -> Set {
    let mut set_cmd = Set::builder()
        .private_key(private_key.to_bytes())
        .listen_port(0u16)
        .replace_peers()
        .build();

    #[cfg(target_os = "linux")]
    {
        set_cmd.fwmark = fwmark;
    }

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

    set_cmd
        .peers
        .push(SetPeer::builder().peer(boring_peer).build());

    set_cmd
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

    // FIXME: mtu is not set

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
