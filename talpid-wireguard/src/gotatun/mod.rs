use crate::{
    Tunnel, TunnelError,
    config::Config,
    obfuscation::RunningObfuscation,
    stats::{Stats, StatsMap},
};
use gotatun::{
    device::{Device, DeviceBuilder, DeviceTransports},
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::{
        IpRecv,
        channel::{TunChannelRx, TunChannelTx},
        tun_async_device::TunDevice as GotaTunDevice,
    },
    udp::channel::{UdpChannelFactory, new_udp_tun_channel},
    x25519::StaticSecret,
};
#[cfg(not(target_os = "android"))]
use ipnetwork::IpNetwork;
#[cfg(target_os = "android")]
use std::os::fd::IntoRawFd;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::Deref,
    sync::{Arc, Mutex},
};
use talpid_net::bypass::SocketBypass;
use talpid_tunnel::tun_provider::{self, Tun, TunProvider};
use talpid_tunnel_config_client::DaitaSettings;
use talpid_types::net::{obfuscation::LwoVersion, wireguard::PeerConfig};
use tun::{AbstractDevice, AsyncDevice};

#[cfg(all(feature = "multihop-pcap", target_os = "linux"))]
use gotatun::tun::{
    IpSend,
    pcap::{PcapSniffer, PcapStream},
};

mod conversions;
mod obfuscation;
mod source_filter;

use conversions::to_gotatun_peer;
pub use obfuscation::{MaybeObfuscatingTransportFactory, lwo_timer_params, lwo_version};
use source_filter::SourceFilter;

type TransportFactory = MaybeObfuscatingTransportFactory;

/// Everything read from the TUN device passes the source filter before it enters the tunnel.
type TunRx = SourceFilter<GotaTunDevice>;

type SinglehopDevice = Device<(TransportFactory, GotaTunDevice, TunRx)>;
type ExitDevice = Device<(UdpChannelFactory, GotaTunDevice, TunRx)>;

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

    /// Name of the tun interface.
    interface_name: String,
}

impl GotaTun {
    async fn new(
        tun_dev: AsyncDevice,
        bypass: Arc<dyn SocketBypass>,
        obfuscation: Option<RunningObfuscation>,
        config: &Config,
        daita: Option<&DaitaSettings>,
        interface_name: String,
    ) -> Result<Self, TunnelError> {
        let tun_dev = GotaTunDevice::from_tun_device(tun_dev)
            .map_err(|e| TunnelError::RecoverableStartWireguardError(Box::new(e)))?;

        let obfuscation = obfuscation.map(|obfuscation| {
            obfuscation.with_client_public_key(config.tunnel.private_key.public_key())
        });
        let devices = create_devices(config, daita, tun_dev, bypass, obfuscation)
            .await
            .map_err(TunnelError::GotaTunDevice)?;

        Ok(Self {
            interface_name,
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
    /// Configure the gotatun device.
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
    /// Configure gotatun devices.
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

/// A tunnel device that is opened for a GotaTun tunnel, see [`start_gotatun`].
pub struct OpenedTun {
    device: AsyncDevice,
    interface_name: String,
    #[cfg(target_os = "android")]
    is_new: bool,
}

impl OpenedTun {
    /// Whether the tunnel device was created, rather than reused. Android applies the routes of a
    /// new tunnel device asynchronously.
    #[cfg(target_os = "android")]
    pub fn is_new(&self) -> bool {
        self.is_new
    }
}

/// Open the tunnel device for a GotaTun tunnel.
pub fn open_tun(
    config: &Config,
    tun_provider: Arc<Mutex<tun_provider::TunProvider>>,
) -> super::Result<OpenedTun> {
    log::trace!("calling get_tunnel_for_userspace");

    #[cfg(not(target_os = "android"))]
    {
        let tun = get_tunnel_for_userspace(tun_provider, config, config.get_tunnel_destinations())?;
        #[cfg(unix)]
        let device = tun.into_inner().into_inner();
        #[cfg(windows)]
        let device = tun.into_inner();

        let interface_name = device.deref().tun_name().unwrap();
        Ok(OpenedTun {
            device,
            interface_name,
        })
    }

    #[cfg(target_os = "android")]
    {
        let (tun, fd) = get_tunnel_for_userspace(tun_provider, config)?;

        let mut tun_config = tun::Configuration::default();
        tun_config.raw_fd(fd);

        let mut device = tun::Device::new(&tun_config).unwrap();

        // HACK: the `tun` crate does not implement AbstractDevice::(set_)mtu on Android, instead
        // they are stubbed. `mtu()` will simply return the value set by `set_mtu()`, or 1500.
        //
        // GotaTun will try to read the MTU from this, so call set_mtu here with the correct value.
        device.set_mtu(config.mtu).unwrap();

        let device = tun::AsyncDevice::new(device).unwrap();
        let interface_name = device.deref().tun_name().unwrap();
        Ok(OpenedTun {
            device,
            interface_name,
            is_new: tun.is_new,
        })
    }
}

/// Start a GotaTun tunnel on `tun`.
pub async fn start_gotatun(
    tun: OpenedTun,
    config: &Config,
    daita: Option<&DaitaSettings>,
    obfuscation: Option<RunningObfuscation>,
    bypass: Arc<dyn SocketBypass>,
) -> super::Result<GotaTun> {
    log::info!("GotaTun::start_tunnel");

    log::trace!("passing tunnel dev to gotatun");
    let gotatun = GotaTun::new(
        tun.device,
        bypass,
        obfuscation,
        config,
        daita,
        tun.interface_name,
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

/// Open a tunnel device and start a GotaTun tunnel on it.
#[cfg(not(target_os = "android"))]
pub async fn open_gotatun_tunnel(
    config: &Config,
    daita: Option<&DaitaSettings>,
    obfuscation: Option<RunningObfuscation>,
    tun_provider: Arc<Mutex<tun_provider::TunProvider>>,
    bypass: Arc<dyn SocketBypass>,
) -> super::Result<GotaTun> {
    let tun = open_tun(config, tun_provider)?;
    start_gotatun(tun, config, daita, obfuscation, bypass).await
}

/// Configure a gotatun entry or singlehop device
async fn configure_entry_device(
    device: &Device<impl DeviceTransports>,
    config: &Config,
    daita: Option<&DaitaSettings>,
) -> Result<(), gotatun::device::Error> {
    let private_key = StaticSecret::from(config.tunnel.private_key.to_bytes());
    let mut entry_peer = to_gotatun_peer(&config.entry_peer, daita);

    if obfuscation::lwo_version(config) == Some(LwoVersion::V2) {
        entry_peer = entry_peer.dangerously_with_timer_params(obfuscation::lwo_timer_params());
    }

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
}

/// Create and configure gotatun devices.
///
/// Will create an [EntryDevice] and an [ExitDevice] if `config` is a multihop config,
/// and a [SinglehopDevice] otherwise.
async fn create_devices(
    config: &Config, // TODO: do not include config to reduce confusion
    daita: Option<&DaitaSettings>,
    tun_dev: GotaTunDevice,
    bypass: Arc<dyn SocketBypass>,
    obfuscation: Option<RunningObfuscation>,
) -> Result<Devices, gotatun::device::Error> {
    async fn create_devices_inner(
        config: &Config, // TODO: do not include config to reduce confusion
        daita: Option<&DaitaSettings>,
        tun_dev: GotaTunDevice,
        bypass: Arc<dyn SocketBypass>,
        obfuscation: Option<RunningObfuscation>,
        optimize_buffer_size: bool,
    ) -> Result<Devices, gotatun::device::Error> {
        let factory = MaybeObfuscatingTransportFactory::new(
            optimize_buffer_size,
            obfuscation,
            config.entry_peer.endpoint,
            bypass,
        );
        // The addresses assigned to the tun device, i.e. the only source addresses we accept
        // packets from. See [SourceFilter].
        let source_v4 = config.tunnel_ipv4();
        let source_v6 = config.tunnel_ipv6();

        let devices = if let Some(exit_peer) = &config.exit_peer {
            // Multihop setup

            // Calculate length of extra headers, assuming no optional header fields (i.e. IP
            // options)
            let multihop_overhead = match exit_peer.endpoint.ip() {
                IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
                IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
            };

            let exit_mtu = tun_dev.mtu();
            let entry_mtu = exit_mtu.increase(multihop_overhead as u16).unwrap(/* TODO: this can happen if tun mtu is max i think*/);

            let (tun_channel_tx, tun_channel_rx, udp_channels) = new_udp_tun_channel(
                PACKET_CHANNEL_CAPACITY,
                source_v4.unwrap_or(Ipv4Addr::UNSPECIFIED),
                source_v6.unwrap_or(Ipv6Addr::UNSPECIFIED),
                entry_mtu,
            );

            let tun_rx = SourceFilter::new(tun_dev.clone(), source_v4, source_v6);
            let exit_device = DeviceBuilder::new()
                .with_udp(udp_channels)
                .with_ip_pair(tun_dev, tun_rx)
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

            let tun_rx = SourceFilter::new(tun_dev.clone(), source_v4, source_v6);
            let device = DeviceBuilder::new()
                .with_udp(factory)
                .with_ip_pair(tun_dev, tun_rx)
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
        bypass.clone(),
        obfuscation.clone(),
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
            create_devices_inner(config, daita, tun_dev, bypass, obfuscation, false).await
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
