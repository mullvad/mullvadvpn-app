#[cfg(target_os = "android")]
use crate::config::patch_allowed_ips;
use crate::{
    Tunnel, TunnelError,
    config::Config,
    stats::{Stats, StatsMap},
};
#[cfg(target_os = "android")]
use gotatun::udp::UdpTransportFactory;
use gotatun::{
    device::{Device, DeviceBuilder, DeviceTransports},
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
    x25519::StaticSecret,
};
#[cfg(target_os = "android")]
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{UdpRecv, UdpSend},
};
#[cfg(not(target_os = "android"))]
use ipnetwork::IpNetwork;
#[cfg(target_os = "android")]
use nix::sys::socket::{MsgFlags, MultiHeaders, SockaddrStorage};
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::Deref,
    sync::{Arc, Mutex},
};
#[cfg(target_os = "android")]
use std::{
    io::{self, IoSlice},
    os::fd::{AsFd, AsRawFd, BorrowedFd, IntoRawFd},
};
use talpid_tunnel::tun_provider::{self, Tun, TunProvider};
use talpid_tunnel_config_client::DaitaSettings;
use talpid_types::net::wireguard::PeerConfig;
use tun::{AbstractDevice, AsyncDevice};

#[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
use gotatun::tun::{
    IpSend,
    pcap::{PcapSniffer, PcapStream},
};

mod conversions;
mod obfuscation;

use conversions::to_gotatun_peer;
use obfuscation::MaybeObfuscatingTransportFactory;

#[cfg(target_os = "android")]
const ANDROID_UDP_MAX_PACKET_COUNT: usize = 100;

#[cfg(target_os = "android")]
type UdpFactory = AndroidUdpSocketFactory;

#[cfg(not(target_os = "android"))]
type UdpFactory = UdpSocketFactory;

type TransportFactory = MaybeObfuscatingTransportFactory<UdpFactory>;

type SinglehopDevice = Device<(TransportFactory, GotaTunDevice, GotaTunDevice)>;
type ExitDevice = Device<(UdpChannelFactory, GotaTunDevice, GotaTunDevice)>;

#[cfg(not(all(feature = "multihop-pcap", target_os = "linux")))]
type EntryDevice = Device<(TransportFactory, TunChannelTx, TunChannelRx)>;
#[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
type EntryDevice = Device<(
    TransportFactory,
    PcapSniffer<TunChannelTx>,
    PcapSniffer<TunChannelRx>,
)>;

const PACKET_CHANNEL_CAPACITY: usize = 100;

pub struct GotaTun {
    /// Device handles
    /// INVARIANT: Must always be `Some`.
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
        .await
        .map_err(TunnelError::GotaTunDevice)?;

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
    Singlehop(Singlehop),
    Multihop(Multihop),
}

struct Singlehop {
    device: SinglehopDevice,
}

impl Singlehop {
    /// (Re)Configure the gotatun device.
    async fn configure(
        &mut self,
        config: &Config,
        daita: Option<&DaitaSettings>,
    ) -> Result<(), gotatun::device::Error> {
        log::trace!(
            "configuring gotatun singlehop device (daita={})",
            daita.is_some()
        );
        configure_entry_device(&self.device, config, daita).await
    }

    async fn stop(self) {
        self.device.stop().await
    }
}

struct Multihop {
    entry_device: EntryDevice,
    exit_device: ExitDevice,
}

impl Multihop {
    /// (Re)Configure gotatun devices.
    ///
    /// Take precaution to ensure that `exit_peer` is linked to `config`.
    async fn configure(
        &mut self,
        config: &Config,
        // TODO: Express relationship between `config` and `exit_peer` at the type level.
        exit_peer: &PeerConfig,
        daita: Option<&DaitaSettings>,
    ) -> Result<(), gotatun::device::Error> {
        let private_key = StaticSecret::from(config.tunnel.private_key.to_bytes());
        let exit_peer = to_gotatun_peer(exit_peer, None);

        log::trace!(
            "configuring gotatun multihop device (daita={})",
            daita.is_some()
        );

        configure_entry_device(&self.entry_device, config, daita).await?;
        configure_exit_device(&self.exit_device, private_key, exit_peer).await?;

        Ok(())
    }

    async fn stop(self) {
        let Multihop {
            entry_device,
            exit_device,
        } = self;
        exit_device.stop().await;
        entry_device.stop().await;
    }
}

impl Devices {
    async fn stop(self) {
        match self {
            Devices::Singlehop(device) => device.stop().await,
            Devices::Multihop(devices) => devices.stop().await,
        }
    }
}

#[cfg(target_os = "android")]
struct AndroidUdpSocketFactory {
    pub tun: Arc<Tun>,
    pub udp: UdpSocketFactory,
}

#[cfg(target_os = "android")]
impl UdpTransportFactory for AndroidUdpSocketFactory {
    type SendV4 = AndroidUdpSocket;
    type SendV6 = AndroidUdpSocket;
    type RecvV4 = AndroidUdpSocket;
    type RecvV6 = AndroidUdpSocket;

    async fn bind(
        &mut self,
        params: &gotatun::udp::UdpTransportFactoryParams,
    ) -> std::io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        let (udp_v4_tx, udp_v6_tx) = bind_android_udp_sockets(params, &self.udp)?;

        self.tun
            .bypass(&udp_v4_tx)
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.tun
            .bypass(&udp_v6_tx)
            .map_err(|error| io::Error::other(error.to_string()))?;

        let udp_v4_rx = udp_v4_tx.clone();
        let udp_v6_rx = udp_v6_tx.clone();

        Ok(((udp_v4_tx, udp_v4_rx), (udp_v6_tx, udp_v6_rx)))
    }
}

#[cfg(target_os = "android")]
#[derive(Clone)]
struct AndroidUdpSocket {
    inner: Arc<tokio::net::UdpSocket>,
}

#[cfg(target_os = "android")]
impl AndroidUdpSocket {
    fn bind(addr: std::net::SocketAddr, factory: &UdpSocketFactory) -> io::Result<Self> {
        let domain = match addr {
            std::net::SocketAddr::V4(..) => socket2::Domain::IPV4,
            std::net::SocketAddr::V6(..) => socket2::Domain::IPV6,
        };

        let socket =
            socket2::Socket::new(domain, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))?;
        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;

        if let Some(recv_buffer_size) = factory.recv_buffer_size
            && let Err(error) = socket.set_recv_buffer_size(recv_buffer_size)
        {
            if cfg!(debug_assertions) {
                return Err(error);
            }
            log::error!("Failed to change UDP socket receive buffer size: {error}");
        }

        if let Some(send_buffer_size) = factory.send_buffer_size
            && let Err(error) = socket.set_send_buffer_size(send_buffer_size)
        {
            if cfg!(debug_assertions) {
                return Err(error);
            }
            log::error!("Failed to change UDP socket send buffer size: {error}");
        }

        socket.bind(&socket2::SockAddr::from(addr))?;
        let socket = tokio::net::UdpSocket::from_std(socket.into())?;

        Ok(Self {
            inner: Arc::new(socket),
        })
    }

    fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.inner.local_addr()
    }
}

#[cfg(target_os = "android")]
impl AsFd for AndroidUdpSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(target_os = "android")]
impl UdpSend for AndroidUdpSocket {
    type SendManyBuf = AndroidSendmmsgBuf;

    async fn send_to(&self, packet: Packet, destination: std::net::SocketAddr) -> io::Result<()> {
        self.inner.send_to(&packet, destination).await?;
        Ok(())
    }

    async fn send_many_to(
        &self,
        buf: &mut AndroidSendmmsgBuf,
        packets: &mut Vec<(Packet, std::net::SocketAddr)>,
    ) -> io::Result<()> {
        debug_assert!(packets.len() <= ANDROID_UDP_MAX_PACKET_COUNT);
        if packets.len() > ANDROID_UDP_MAX_PACKET_COUNT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "send_many_to: Number of packets may not exceed {ANDROID_UDP_MAX_PACKET_COUNT}"
                ),
            ));
        }

        let socket = &self.inner;
        let fd = socket.as_raw_fd();

        buf.targets.clear();

        let mut packets_buf = [[IoSlice::new(&[])]; ANDROID_UDP_MAX_PACKET_COUNT];
        for ((packet, target), packets_buf) in packets.iter().zip(&mut packets_buf) {
            buf.targets.push(Some(SockaddrStorage::from(*target)));
            *packets_buf = [IoSlice::new(&packet[..])];
        }

        let len = buf.targets.len();
        let pkts = &packets_buf[..len];
        let mut packet_buf_start = 0;
        while packet_buf_start < len {
            let result = socket
                .async_io(tokio::io::Interest::WRITABLE, || {
                    let mut multiheaders =
                        MultiHeaders::preallocate(pkts[packet_buf_start..].len(), None);
                    let multiresult = nix::sys::socket::sendmmsg(
                        fd,
                        &mut multiheaders,
                        &pkts[packet_buf_start..],
                        &buf.targets[packet_buf_start..],
                        [],
                        MsgFlags::MSG_DONTWAIT,
                    )?;
                    Ok(multiresult.count())
                })
                .await;
            let n = result?;
            packet_buf_start += n;
        }

        packets.clear();
        Ok(())
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        ANDROID_UDP_MAX_PACKET_COUNT
    }

    fn local_addr(&self) -> io::Result<Option<std::net::SocketAddr>> {
        AndroidUdpSocket::local_addr(self).map(Some)
    }
}

#[cfg(target_os = "android")]
#[derive(Default)]
struct AndroidSendmmsgBuf {
    targets: Vec<Option<SockaddrStorage>>,
}

#[cfg(target_os = "android")]
impl UdpRecv for AndroidUdpSocket {
    type RecvManyBuf = ();

    async fn recv_from(
        &mut self,
        pool: &mut PacketBufPool,
    ) -> io::Result<(Packet, std::net::SocketAddr)> {
        let mut packet = pool.get();
        let (length, source) = self.inner.recv_from(&mut packet).await?;
        packet.truncate(length);
        Ok((packet, source))
    }
}

#[cfg(target_os = "android")]
fn bind_android_udp_sockets(
    params: &gotatun::udp::UdpTransportFactoryParams,
    factory: &UdpSocketFactory,
) -> io::Result<(AndroidUdpSocket, AndroidUdpSocket)> {
    let udp_v4 = AndroidUdpSocket::bind((params.addr_v4, params.port).into(), factory)?;

    match params.port {
        0 => bind_android_ipv6_with_retry(params.addr_v4, params.addr_v6, udp_v4, factory),
        port => {
            let udp_v6 = AndroidUdpSocket::bind((params.addr_v6, port).into(), factory)?;
            Ok((udp_v4, udp_v6))
        }
    }
}

#[cfg(target_os = "android")]
fn bind_android_ipv6_with_retry(
    addr_v4: Ipv4Addr,
    addr_v6: Ipv6Addr,
    mut udp_v4: AndroidUdpSocket,
    factory: &UdpSocketFactory,
) -> io::Result<(AndroidUdpSocket, AndroidUdpSocket)> {
    const MAX_RETRIES: u32 = 10;

    let mut port = udp_v4.local_addr()?.port();
    let mut retries = 0u32;

    let udp_v6 = loop {
        match AndroidUdpSocket::bind((addr_v6, port).into(), factory) {
            Ok(socket) => break socket,
            Err(error) if error.kind() == io::ErrorKind::AddrInUse && retries < MAX_RETRIES => {
                retries += 1;
                log::debug!("IPv6 port {port} already in use, retrying ({retries}/{MAX_RETRIES})");
                udp_v4 = AndroidUdpSocket::bind((addr_v4, 0).into(), factory)?;
                port = udp_v4.local_addr()?.port();
            }
            Err(error) => return Err(error),
        }
    };

    Ok((udp_v4, udp_v6))
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

        // There is a brief period of time between setting up a Wireguard tunnel and the tunnel being ready to serve
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

        let mut tun_config = tun::Configuration::default();
        tun_config.raw_fd(fd);

        let mut device = tun::Device::new(&tun_config).unwrap();

        // HACK: the `tun` crate does not implement AbstractDevice::(set_)mtu on Android, instead
        // they are stubbed. `mtu()` will simply return the value set by `set_mtu()`, or 1500.
        //
        // GotaTun will try to read the MTU from this, so call set_mtu here with the correct value.
        device.set_mtu(config.mtu).unwrap();

        (Arc::new(tun), tun::AsyncDevice::new(device).unwrap())
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

/// Configure a gotatun entry or singlehop device
async fn configure_entry_device(
    device: &Device<impl DeviceTransports>,
    config: &Config,
    daita: Option<&DaitaSettings>,
) -> Result<(), gotatun::device::Error> {
    let private_key = StaticSecret::from(config.tunnel.private_key.to_bytes());
    let entry_peer = to_gotatun_peer(&config.entry_peer, daita);
    device
        .write(async |device| {
            device.clear_peers();
            device.set_private_key(private_key).await;
            device.add_peer(entry_peer);
            #[cfg(target_os = "linux")]
            if let Some(fwmark) = config.fwmark {
                device.set_fwmark(fwmark)?;
            }
            Ok(())
        })
        .await
        .flatten()
        .inspect_err(|err| {
            log::error!("Failed to set gotatun config: {err:#}");
        })
}

/// Configure gotatun exit device
async fn configure_exit_device(
    device: &Device<impl DeviceTransports>,
    private_key: StaticSecret,
    exit_peer: gotatun::device::Peer,
) -> Result<(), gotatun::device::Error> {
    device
        .write(async |device| {
            device.clear_peers();
            device.set_private_key(private_key).await;
            device.add_peer(exit_peer);
        })
        .await
        .inspect_err(|err| {
            log::error!("Failed to set gotatun config: {err:#}");
        })
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
        /// Read all peer stats from a gotatun [`Device`].
        async fn get_stats(device: &Device<impl DeviceTransports>) -> StatsMap {
            let peers = device.read(async |device| device.peers().await).await;

            peers
                .into_iter()
                .map(|peer| {
                    let public_key = peer.peer.public_key.to_bytes();
                    let stats = Stats::from(peer.stats);
                    (public_key, stats)
                })
                .collect()
        }

        let stats = match self.devices.as_ref() {
            Some(Devices::Singlehop(Singlehop { device })) => get_stats(device).await,
            Some(Devices::Multihop(Multihop {
                entry_device,
                exit_device,
            })) => {
                let mut stats = get_stats(entry_device).await;
                stats.extend(get_stats(exit_device).await);
                stats
            }
            None if cfg!(debug_assertions) => unreachable!("device must be Some"),
            None => StatsMap::default(),
        };

        Ok(stats)
    }

    fn set_config<'a>(
        &'a mut self,
        config: Config,
        daita: Option<DaitaSettings>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TunnelError>> + Send + 'a>> {
        Box::pin(async move {
            self.config = config;
            // If we're switching to/from multihop, we'll need to tear down the old device(s)
            // and set them up with the new DeviceTransports
            let devices = match self.devices.take() {
                // Switching from singlehop to multihop
                Some(Devices::Singlehop(device))
                    if let Some(_exit_peer) = self.config.exit_peer.as_ref() =>
                {
                    // Tear down old device and recreate new multihop devices.
                    device.stop().await;
                    create_devices(
                        &self.config,
                        daita.as_ref(),
                        self.tun_dev.clone(),
                        #[cfg(target_os = "android")]
                        self.android_tun.clone(),
                    )
                    .await
                    .map_err(TunnelError::GotaTunDevice)?
                }
                // FIXME: When re-configuring a device with a new `psk`, `gotatun` seemingly
                // borks out. A known workaround is to tear down the old device and set up a new
                // one.
                Some(Devices::Singlehop(device))
                    if let Some(_psk) = self.config.entry_peer.psk.as_ref() =>
                {
                    // Tear down old device and recreate new multihop devices.
                    device.stop().await;
                    create_devices(
                        &self.config,
                        daita.as_ref(),
                        self.tun_dev.clone(),
                        #[cfg(target_os = "android")]
                        self.android_tun.clone(),
                    )
                    .await
                    .map_err(TunnelError::GotaTunDevice)?
                }
                // Simply reconfigure the singlehop device.
                Some(Devices::Singlehop(mut device)) => {
                    device
                        .configure(&self.config, daita.as_ref())
                        .await
                        .map_err(TunnelError::GotaTunDevice)?;
                    Devices::Singlehop(device)
                }
                // Simply reconfigure the multihop devices.
                Some(Devices::Multihop(mut devices))
                    if let Some(exit_peer) = self.config.exit_peer.as_ref() =>
                {
                    devices
                        .configure(&self.config, exit_peer, daita.as_ref())
                        .await
                        .map_err(TunnelError::GotaTunDevice)?;
                    Devices::Multihop(devices)
                }
                // Switching from multihop to singlehop
                Some(Devices::Multihop(devices)) => {
                    // Tear down old devices and recreate new singlehop device.
                    devices.stop().await;
                    create_devices(
                        &self.config,
                        daita.as_ref(),
                        self.tun_dev.clone(),
                        #[cfg(target_os = "android")]
                        self.android_tun.clone(),
                    )
                    .await
                    .map_err(TunnelError::GotaTunDevice)?
                }
                None => create_devices(
                    &self.config,
                    daita.as_ref(),
                    self.tun_dev.clone(),
                    #[cfg(target_os = "android")]
                    self.android_tun.clone(),
                )
                .await
                .map_err(TunnelError::GotaTunDevice)?,
            };

            self.devices = Some(devices);
            Ok(())
        })
    }
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
) -> Result<Devices, gotatun::device::Error> {
    async fn create_devices_inner(
        config: &Config, // TODO: do not include config to reduce confusion
        daita: Option<&DaitaSettings>,
        tun_dev: GotaTunDevice,
        #[cfg(target_os = "android")] android_tun: Arc<Tun>,
        optimize_buffer_size: bool,
    ) -> Result<Devices, gotatun::device::Error> {
        let factory = udp_obfuscator_factory(
            config,
            optimize_buffer_size,
            #[cfg(target_os = "android")]
            android_tun,
        );
        let devices = if let Some(exit_peer) = &config.exit_peer {
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

            let exit_device = DeviceBuilder::new()
                .with_udp(udp_channels)
                .with_ip(tun_dev)
                .build()
                .await?;

            // Hacky way of dumping entry<->exit traffic to a unix socket which wireshark can read.
            // See docs on wrap_in_pcap_sniffer for an explanation.
            #[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
            let (tun_channel_tx, tun_channel_rx) =
                wrap_in_pcap_sniffer(tun_channel_tx, tun_channel_rx);

            let entry_device = DeviceBuilder::new()
                .with_udp(factory)
                .with_ip_pair(tun_channel_tx, tun_channel_rx)
                .build()
                .await?;
            let mut devices = Multihop {
                entry_device,
                exit_device,
            };
            devices.configure(config, exit_peer, daita).await?;
            Devices::Multihop(devices)
        } else {
            // Singlehop setup

            let device = DeviceBuilder::new()
                .with_udp(factory)
                .with_ip(tun_dev)
                .build()
                .await?;
            let mut device = Singlehop { device };
            device.configure(config, daita).await?;
            Devices::Singlehop(device)
        };

        Ok(devices)
    }

    match create_devices_inner(
        config,
        daita,
        tun_dev.clone(),
        #[cfg(target_os = "android")]
        android_tun.clone(),
        true,
    )
    .await
    {
        Ok(devices) => Ok(devices),
        // Empirically, creating devices may fail when binding the UDP socket due to
        // us wanting to tweak the UDP socket buffer sizes to a larger value than
        // the OS default (suspected hardware related issues / limitations). In that
        // case, `os error 55 ("No buffer space available")`  has been observed.
        //
        // Try to bind UDP sockets with default buffer sizes.
        #[cfg(unix)]
        Err(ref err @ gotatun::device::Error::Bind(ref io_err, _))
            if let Some(errno) = io_err.raw_os_error()
                && nix::errno::Errno::from_raw(errno) == nix::errno::Errno::ENOBUFS =>
        {
            log::error!("Failed to bind UDP socket - retrying with default buffer sizes");
            create_devices_inner(
                config,
                daita,
                tun_dev,
                #[cfg(target_os = "android")]
                android_tun.clone(),
                false,
            )
            .await
        }
        Err(err) => Err(err),
    }
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

    // Route everything into the tunnel and have WireGuard act as a firewall when
    // blocking. These will not necessarily be the actual routes used by android. Those will
    // be generated at a later stage e.g. if Local Network Sharing is enabled.
    // If IPv6 is not enabled in the tunnel we should not route IPv6 traffic as this
    // leads to leaks.
    tun_config.routes = if config.ipv6_gateway.is_some() {
        vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()]
    } else {
        vec!["0.0.0.0/0".parse().unwrap()]
    };

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

/// Provide a [`UdpSocketFactory`] for the entry-device.
///
/// - `optimize_buffer_size`: if UDP socket buffer sizes should be tweaked. Empirically this might
///   now always succeed due to suspected hardware related issues / limitations.
fn udp_obfuscator_factory(
    config: &Config,
    optimize_buffer_size: bool,
    #[cfg(target_os = "android")] android_tun: Arc<Tun>,
) -> MaybeObfuscatingTransportFactory<UdpFactory> {
    let factory = cfg_select! {
        target_os = "android" => {
            AndroidUdpSocketFactory {
                tun: android_tun,
                udp: udp_socket_factory(optimize_buffer_size),
            }
        },
        _ => { udp_socket_factory(optimize_buffer_size) }
    };
    MaybeObfuscatingTransportFactory::from_config(factory, config)
}

/// Provide a [`UdpSocketFactory`] for the entry-device.
///
/// - `optimize_buffer_size`: if UDP socket buffer sizes should be tweaked.
///   This could be beneficial for performance reasons.
#[inline(always)]
fn udp_socket_factory(optimize_buffer_size: bool) -> UdpSocketFactory {
    /// See [`DeviceBuilder::udp_send_buffer_size`] for details.
    const UDP_SEND_BUFFER_SIZE: usize = 7 * 1024 * 1024; // 7 MB (mirror the default of `gotatun-cli`)
    /// See [`DeviceBuilder::udp_recv_buffer_size`] for details.
    const UDP_RECV_BUFFER_SIZE: usize = 7 * 1024 * 1024;

    if optimize_buffer_size {
        UdpSocketFactory {
            recv_buffer_size: Some(UDP_RECV_BUFFER_SIZE),
            send_buffer_size: Some(UDP_SEND_BUFFER_SIZE),
        }
    } else {
        UdpSocketFactory::default()
    }
}
