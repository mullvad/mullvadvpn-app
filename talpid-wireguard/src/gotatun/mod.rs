#[cfg(target_os = "android")]
use crate::config::patch_allowed_ips;
use crate::{
    Tunnel, TunnelError,
    config::Config,
    stats::{DaitaStats, Stats, StatsMap},
};
#[cfg(target_os = "android")]
use gotatun::udp::UdpTransportFactory;
use gotatun::{
    device::{
        DeviceConfig, DeviceHandle,
        api::{ApiClient, ApiServer, command::*},
        peer::AllowedIP,
    },
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::{
        IpRecv,
        channel::{TunChannelRx, TunChannelTx},
        tun_async_device::TunDevice as GotaTunDevice,
    },
    udp::{
        channel::{UdpChannelFactory, new_udp_tun_channel},
        socket::UdpSocketFactory,
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

#[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
use gotatun::tun::{
    IpSend,
    pcap::{PcapSniffer, PcapStream},
};

#[cfg(target_os = "android")]
type UdpFactory = AndroidUdpSocketFactory;

#[cfg(not(target_os = "android"))]
type UdpFactory = UdpSocketFactory;

type SinglehopDevice = DeviceHandle<(UdpFactory, GotaTunDevice)>;
type ExitDevice = DeviceHandle<(UdpChannelFactory, GotaTunDevice)>;

#[cfg(not(all(feature = "multihop-pcap", target_os = "linux")))]
type EntryDevice = DeviceHandle<(UdpFactory, TunChannelTx, TunChannelRx)>;
#[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
type EntryDevice = DeviceHandle<(
    UdpFactory,
    PcapSniffer<TunChannelTx>,
    PcapSniffer<TunChannelRx>,
)>;

const PACKET_CHANNEL_CAPACITY: usize = 100;

pub struct GotaTun {
    /// Device handles
    // TODO: Can we not store this in an option?
    devices: Option<Devices>,

    tun_dev: GotaTunDevice,

    #[cfg(target_os = "android")]
    android_tun: Arc<Tun>,

    /// Tunnel config
    config: Config,

    /// Name of the tun interface.
    interface_name: String,
}

impl GotaTun {
    async fn new(
        tun_dev: AsyncDevice,
        #[cfg(target_os = "android")] android_tun: Arc<Tun>,
        config: Config,
        interface_name: String,
    ) -> Result<Self, TunnelError> {
        let tun_dev = GotaTunDevice::from_tun_device(tun_dev)
            .map_err(|e| TunnelError::RecoverableStartWireguardError(Box::new(e)))?;

        let devices = create_devices(
            &config,
            None,
            tun_dev.clone(),
            #[cfg(target_os = "android")]
            android_tun.clone(),
        )
        .await?;

        Ok(Self {
            config,
            interface_name,
            tun_dev,
            #[cfg(target_os = "android")]
            android_tun,
            devices: Some(devices),
        })
    }
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

impl Devices {
    async fn stop(self) {
        match self {
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
    }
}

/// Errors that can happen when setting up / restarting / reconfiguring GotaTun devices.
#[derive(thiserror::Error, Debug)]
pub enum ConfigureGotaTunDeviceError {
    #[error("Multihop devices were provided with a single config")]
    ExpectedSinglehopDevice,
    #[error("Single devices were provided with a multihop config")]
    ExpectedMultihopDevice,
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
        params: &gotatun::udp::UdpTransportFactoryParams,
    ) -> std::io::Result<((Self::Send, Self::RecvV4), (Self::Send, Self::RecvV6))> {
        let ((udp_v4_tx, udp_v4_rx), (udp_v6_tx, udp_v6_rx)) =
            UdpSocketFactory.bind(params).await?;

        self.tun.bypass(&udp_v4_tx).unwrap();
        self.tun.bypass(&udp_v6_tx).unwrap();

        Ok(((udp_v4_tx, udp_v4_rx), (udp_v6_tx, udp_v6_rx)))
    }
}

/// Configure and start a gotatun tunnel.
pub async fn open_gotatun_tunnel(
    config: &Config,
    tun_provider: Arc<Mutex<tun_provider::TunProvider>>,
    #[cfg(target_os = "android")] route_manager_handle: talpid_routing::RouteManagerHandle,
    #[cfg(target_os = "android")] gateway_only: bool,
) -> super::Result<GotaTun> {
    log::info!("GotaTun::start_tunnel");
    let routes = config.get_tunnel_destinations();

    log::trace!("calling get_tunnel_for_userspace");
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

        let mut device = tun07::Device::new(&tun_config).unwrap();

        // HACK: the `tun` crate does not implement AbstractDevice::(set_)mtu on Android, instead
        // they are stubbed. `mtu()` will simply return the value set by `set_mtu()`, or 1500.
        //
        // GotaTun will try to read the MTU from this, so call set_mtu here with the correct value.
        device.set_mtu(config.mtu).unwrap();

        (Arc::new(tun), tun07::AsyncDevice::new(device).unwrap())
    };

    let interface_name = async_tun.deref().tun_name().unwrap();

    let config = config.clone();
    #[cfg(target_os = "android")]
    let config = match gateway_only {
        // See `wireguard_go` module for why this is needed.
        true => patch_allowed_ips(config),
        false => config,
    };

    log::trace!("passing tunnel dev to gotatun");
    let gotatun = GotaTun::new(
        async_tun,
        #[cfg(target_os = "android")]
        tun.clone(),
        config,
        interface_name,
    )
    .await
    .inspect_err(|e| log::error!("Failed to open GotaTun: {e:?}"))?;

    log::info!(
        r#"This tunnel was brought to you by...
           _______   _ __       ______
          / ____(_)_(_) /_____ /_  __/_  ______
         / / __/ __ \/ __/ __ `// / / / / / __ \
        / /_/ / /_/ / /_/ /_/ // / / /_/ / / / /
        \____/\____/\__/\__,_//_/  \__,_/_/ /_/"#
    );

    Ok(gotatun)
}

/// Create and configure gotatun devices.
///
/// Will create an [EntryDevice] and an [ExitDevice] if `config` is a multihop config,
/// and a [SinglehopDevice] otherwise.
async fn create_devices(
    config: &Config, // TODO: do not include config to reduce confusion
    daita: Option<&DaitaSettings>,
    tun_dev: GotaTunDevice,
    #[cfg(target_os = "android")] android_tun: Arc<Tun>,
) -> Result<Devices, TunnelError> {
    let (entry_api, entry_api_server) = ApiServer::new();
    let gotatun_entry_config = DeviceConfig {
        api: Some(entry_api_server),
    };

    #[cfg(target_os = "android")]
    let udp_factory = AndroidUdpSocketFactory { tun: android_tun };

    #[cfg(not(target_os = "android"))]
    let udp_factory = UdpSocketFactory;

    let mut devices = if let Some(exit_peer) = &config.exit_peer {
        // Multihop setup

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

        // Calculate length of extra headers, assuming no optional header fields (i.e. IP options)
        let multihop_overhead = match exit_peer.endpoint.ip() {
            IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
            IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
        };

        let exit_mtu = tun_dev.mtu();
        let entry_mtu = exit_mtu.increase(multihop_overhead as u16).unwrap(/* TODO: this can happen if tun mtu is max i think*/);

        let (tun_channel_tx, tun_channel_rx, udp_channels) =
            new_udp_tun_channel(PACKET_CHANNEL_CAPACITY, source_v4, source_v6, entry_mtu);

        let (exit_api, exit_api_server) = ApiServer::new();
        let exit_device = ExitDevice::new(
            udp_channels,
            tun_dev.clone(),
            tun_dev,
            DeviceConfig {
                api: Some(exit_api_server),
            },
        )
        .await;

        // Hacky way of dumping entry<->exit traffic to a unix socket which wireshark can read.
        // See docs on wrap_in_pcap_sniffer for an explanation.
        #[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
        let (tun_channel_tx, tun_channel_rx) = wrap_in_pcap_sniffer(tun_channel_tx, tun_channel_rx);

        let entry_device = EntryDevice::new(
            udp_factory,
            tun_channel_tx,
            tun_channel_rx,
            gotatun_entry_config,
        )
        .await;

        Devices::Multihop {
            entry_device,
            entry_api,
            exit_device,
            exit_api,
        }
    } else {
        // Singlehop setup

        let device =
            SinglehopDevice::new(udp_factory, tun_dev.clone(), tun_dev, gotatun_entry_config).await;

        Devices::Singlehop {
            device,
            api: entry_api,
        }
    };

    configure_devices(&mut devices, config, daita).await?;

    Ok(devices)
}

/// (Re)Configure gotatun devices.
async fn configure_devices(
    devices: &mut Devices,
    config: &Config,
    daita: Option<&DaitaSettings>,
) -> Result<(), TunnelError> {
    if let Some(exit_peer) = &config.exit_peer {
        log::trace!(
            "configuring gotatun multihop device (daita={})",
            daita.is_some()
        );

        let Devices::Multihop {
            entry_api,
            exit_api,
            ..
        } = devices
        else {
            return Err(TunnelError::ConfigureGotaTunDevice(
                ConfigureGotaTunDeviceError::ExpectedMultihopDevice,
            ));
        };

        let private_key = &config.tunnel.private_key;
        let peer = &config.entry_peer;
        let set_cmd = create_set_command(
            #[cfg(target_os = "linux")]
            config.fwmark,
            private_key,
            peer,
            daita,
        );
        entry_api.send(set_cmd).await.map_err(|err| {
            log::error!("Failed to set gotatun config: {err:#}");
            TunnelError::SetConfigError
        })?;

        let set_cmd = create_set_command(
            #[cfg(target_os = "linux")]
            config.fwmark,
            private_key,
            exit_peer,
            None, // exit peer never has daita
        );
        exit_api.send(set_cmd).await.map_err(|err| {
            log::error!("Failed to set gotatun config: {err:#}");
            TunnelError::SetConfigError
        })?;
    } else {
        log::trace!(
            "configuring gotatun singlehop device (daita={})",
            daita.is_some()
        );

        let Devices::Singlehop { api, .. } = devices else {
            return Err(TunnelError::ConfigureGotaTunDevice(
                ConfigureGotaTunDeviceError::ExpectedSinglehopDevice,
            ));
        };

        let private_key = &config.tunnel.private_key;
        let peer = &config.entry_peer;
        let set_cmd = create_set_command(
            #[cfg(target_os = "linux")]
            config.fwmark,
            private_key,
            peer,
            daita,
        );
        api.send(set_cmd).await.map_err(|err| {
            log::error!("Failed to set gotatun config: {err:#}");
            TunnelError::SetConfigError
        })?;
    }

    Ok(())
}

#[async_trait::async_trait]
impl Tunnel for GotaTun {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn stop(mut self: Box<Self>) -> Result<(), TunnelError> {
        tokio::runtime::Handle::current().block_on(async {
            // TODO: devices should never be None while this GotaTun instance is running.
            debug_assert!(self.devices.is_some());
            if let Some(devices) = self.devices.take() {
                devices.stop().await;
            }
        });
        Ok(())
    }

    async fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        let mut stats = StatsMap::default();

        let apis = match self.devices.as_ref().unwrap() {
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
                    // TODO: Gotatun should probably return a Unix timestamp (like wg-go)
                    Some(SystemTime::now() - Duration::new(handshake_sec, handshake_nsec))
                };

                let daita = || -> Option<DaitaStats> {
                    Some(DaitaStats {
                        tx_padding_bytes: peer.tx_padding_bytes?,
                        tx_padding_packet_bytes: peer.tx_padding_packet_bytes?,
                        rx_padding_bytes: peer.rx_padding_bytes?,
                        rx_padding_packet_bytes: peer.rx_padding_packet_bytes?,
                    })
                };

                stats.insert(
                    peer.peer.public_key.0,
                    Stats {
                        tx_bytes: peer.tx_bytes.unwrap_or_default(),
                        rx_bytes: peer.rx_bytes.unwrap_or_default(),
                        last_handshake_time: last_handshake(),
                        daita: daita(),
                    },
                );
            }
        }

        Ok(stats)
    }

    fn set_config<'a>(
        &'a mut self,
        config: Config,
        daita: Option<DaitaSettings>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TunnelError>> + Send + 'a>> {
        Box::pin(async move {
            self.config = config;

            // if we're switching to/from multihop, we'll need to tear down the old device(s)
            // and set them up with the new DeviceTransports
            // TODO: Debug configure_devices. Currently we need to tear down the old devices
            // after having exchanged tunnel params with the ephemeral peer. Empirically this
            // is true for both DAITA & PQ.
            // let recreate_devices = old_config.is_multihop() != self.config.is_multihop();
            let recreate_devices = true;

            if recreate_devices {
                // TODO: devices should never be None while this GotaTun instance is running.
                debug_assert!(self.devices.is_some());
                if let Some(devices) = self.devices.take() {
                    devices.stop().await;
                }
            }

            match &mut self.devices {
                Some(devices) => {
                    configure_devices(devices, &self.config, daita.as_ref()).await?;
                }
                None => {
                    self.devices = Some(
                        create_devices(
                            &self.config,
                            daita.as_ref(),
                            self.tun_dev.clone(),
                            #[cfg(target_os = "android")]
                            self.android_tun.clone(),
                        )
                        .await?,
                    )
                }
            };

            Ok(())
        })
    }
}

fn create_set_command(
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
    private_key: &talpid_types::net::wireguard::PrivateKey,
    peer: &talpid_types::net::wireguard::PeerConfig,
    daita: Option<&DaitaSettings>,
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

    let mut gota_peer = Peer::builder()
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
        gota_peer.preshared_key = Some(SetUnset::Set((*psk.as_bytes()).into()));
    }

    let mut set_peer = SetPeer::builder().peer(gota_peer).build();

    if let Some(daita) = daita {
        set_peer.daita_settings = Some(gotatun::device::daita::api::DaitaSettings {
            maybenot_machines: daita.client_machines.clone(),
            max_padding_frac: daita.max_padding_frac,
            max_blocking_frac: daita.max_blocking_frac,
            // TODO: tweak to sane values
            max_blocked_packets: 1024,
            min_blocking_capacity: 50,
        });
    }

    set_cmd.peers.push(set_peer);

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

    // Route everything into the tunnel and have WireGuard act as a firewall when
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

/// Wrap `ip_send` and `ip_recv` in [PcapSniffer]s for use with Wireshark.
///
/// With userspace multihop, the [ExitDevice] communicates with the network through the
/// [EntryDevice], without going through the kernel. That means there is no network interface
/// for wireshark to sniff. By interposing [PcapSniffer]s, any packets that are sent to `ip_send`,
/// or received from `ip_recv`, will _also_ be written to a unix socket, encoded using the pcap
/// file format.
///
/// The unix socket can be opened in wireshark to inspect communication with the [ExitDevice]s peer.
/// ```sh
/// wireshark -k -i /tmp/mullvad-multihop.pcap
/// ```
#[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
fn wrap_in_pcap_sniffer<S, R>(ip_send: S, ip_recv: R) -> (PcapSniffer<S>, PcapSniffer<R>)
where
    S: IpSend,
    R: IpRecv,
{
    use std::{
        fs,
        os::unix::{fs::PermissionsExt, net::UnixListener},
        sync::LazyLock,
        time::Instant,
    };

    const SOCKET_PATH: &str = "/tmp/mullvad-multihop.pcap";

    /// The global pcap writer. We initialize it once so that we can re-use the same unix socket
    /// for the entire lifetime of the application.
    static WRITER: LazyLock<PcapStream> = LazyLock::new(|| {
        log::warn!("Binding pcap socket to {SOCKET_PATH:?}");
        let _ = fs::remove_file(SOCKET_PATH);
        let listener = UnixListener::bind(SOCKET_PATH).unwrap();
        let _ = fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o777));

        log::warn!("Waiting for connection to pcap socket");
        log::warn!("    wireshark -k -i {SOCKET_PATH:?}");
        let (stream, _) = listener
            .accept()
            .expect("Error while waiting for pcap listener");

        PcapStream::new(Box::new(stream))
    });

    let start_time = Instant::now();

    let w = WRITER.clone();
    let ip_send = PcapSniffer::new(ip_send, w, start_time);

    let w = WRITER.clone();
    let ip_recv = PcapSniffer::new(ip_recv, w, start_time);

    (ip_send, ip_recv)
}
