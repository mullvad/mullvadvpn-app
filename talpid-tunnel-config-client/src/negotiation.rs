//! Negotiate ephemeral peers through temporary GotaTun devices.
//!
//! The devices reach the config services of the relays through a userspace network stack, so no
//! tunnel device, routes or firewall exceptions are needed. The *ingress relay* is the relay that
//! the devices connect to directly: the only relay in singlehop, and the entry relay in multihop.

use crate::{
    CONFIG_SERVICE_PORT, DaitaSettings, EphemeralPeer, Error, request_ephemeral_peer_over_stream,
};
use gotatun::{
    device::{Device, DeviceBuilder, DeviceTransports, Peer},
    noise::TimerParams,
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::MtuWatcher,
    udp::{UdpTransportFactory, channel::new_udp_tun_channel},
    x25519::StaticSecret,
};
use ipnetwork::IpNetwork;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use talpid_netstack::smoltcp_network::{
    SmoltcpHandle, SmoltcpIpRecv, SmoltcpIpSend, SmoltcpNetworkConfig, SmoltcpNetworkGuard,
    smoltcp_network,
};
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};

/// MTU of the userspace network stack. This is the lowest possible IPv4 MTU, since the path MTU may
/// be lower than the tunnel MTU, and has not been detected yet.
const USERSPACE_NET_MTU: u16 = 576;

/// Capacity of the channels between the entry and exit devices in multihop.
const MULTIHOP_CHANNEL_CAPACITY: usize = 100;

/// How often to check whether a device has completed a handshake with its peer.
const HANDSHAKE_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// A relay to negotiate an ephemeral peer with.
pub struct Relay {
    pub public_key: PublicKey,
    pub endpoint: SocketAddr,
}

impl Relay {
    fn pubkey(&self) -> gotatun::x25519::PublicKey {
        (*self.public_key.as_bytes()).into()
    }
}

/// The relays that the tunnel goes through.
pub enum Relays {
    Singlehop(Relay),
    Multihop { entry: Relay, exit: Relay },
}

impl Relays {
    fn ingress(&self) -> &Relay {
        match self {
            Relays::Singlehop(relay) => relay,
            Relays::Multihop { entry, .. } => entry,
        }
    }
}

/// What to negotiate with the config services in [`negotiate_ephemeral_peers`].
#[derive(Clone, Copy)]
pub struct Negotiables {
    /// Negotiate a PSK with every relay, using a post-quantum safe key exchange.
    pub post_quantum: bool,
    /// Request DAITA machines from the ingress relay.
    pub daita: bool,
}

/// Parameters for [`negotiate_ephemeral_peers`].
pub struct NegotiationConfig {
    /// The WireGuard private key of this device.
    pub private_key: PrivateKey,
    /// The IPv4 address of this device in the tunnel.
    pub tunnel_ipv4: Ipv4Addr,
    /// The address of the config service in the tunnel.
    pub config_service_ip: Ipv4Addr,
    pub relays: Relays,
    /// WireGuard timers to use with the ingress relay, if they differ from the defaults.
    pub ingress_timer_params: Option<TimerParams>,
    /// Time limit for the exchange with each config service.
    pub timeout: Duration,
    /// TCP timeout. See [`talpid_netstack::smoltcp_network::SmoltcpHandle::tcp_connect`].
    pub tcp_timeout: Option<Duration>,
    /// Time limit for the WireGuard handshake with each relay. A relay that does not complete a
    /// handshake in time is unreachable, so there is no point in waiting for [`Self::timeout`].
    pub handshake_timeout: Duration,
    /// Negotiate a different ephemeral key with the exit relay in multihop. This requires a
    /// separate device for each relay, since a device has a single private key.
    pub separate_exit_key: bool,
}

impl NegotiationConfig {
    fn allowed_ip(&self) -> IpNetwork {
        // Only allow tunnel to talk to the config service
        IpNetwork::from(IpAddr::V4(self.config_service_ip))
    }
}

/// Ephemeral peers negotiated by [`negotiate_ephemeral_peers`].
pub struct NegotiatedPeers {
    /// The private key to use with the ingress relay, and with the exit relay unless
    /// [`Self::exit_private_key`] is set.
    pub private_key: PrivateKey,
    /// The private key to use with the exit relay in multihop, if it differs from
    /// [`Self::private_key`]. See [`NegotiationConfig::separate_exit_key`].
    pub exit_private_key: Option<PrivateKey>,
    /// The PSK to use with the ingress relay.
    pub ingress_psk: Option<PresharedKey>,
    /// The PSK to use with the exit relay in multihop.
    pub exit_psk: Option<PresharedKey>,
    /// The DAITA settings to use with the ingress relay.
    pub daita: Option<DaitaSettings>,
}

/// Errors from [`negotiate_ephemeral_peers`].
#[derive(thiserror::Error, Debug)]
pub enum NegotiationError {
    /// A config service did not respond in time.
    #[error("Timed out while negotiating ephemeral peer")]
    Timeout,
    /// Failed to create a GotaTun device.
    #[error("Failed to create GotaTun device")]
    Device(#[source] gotatun::device::Error),
    /// The exchange with a config service failed.
    #[error("Failed to exchange ephemeral peer")]
    Exchange(#[source] Error),
}

/// Negotiate ephemeral peers with the relays in `config`, using a new ephemeral key. In multihop,
/// the exit relay gets a key of its own if [`NegotiationConfig::separate_exit_key`] is set.
///
/// In multihop, the entry peer is negotiated first. The exit peer is then negotiated through the
/// entry relay, using the negotiated entry peer.
///
/// The temporary devices reach the ingress relay through the UDP transports that
/// `ingress_transport` creates for a device with the given public key.
///
/// Each exchange with a config service takes at most [`NegotiationConfig::timeout`], or
/// [`NegotiationConfig::handshake_timeout`] if the relay does not complete a WireGuard handshake.
pub async fn negotiate_ephemeral_peers<F: UdpTransportFactory>(
    config: &NegotiationConfig,
    negotiate: Negotiables,
    ingress_transport: impl Fn(&PublicKey) -> F,
) -> Result<NegotiatedPeers, NegotiationError> {
    let ephemeral_key = PrivateKey::new_from_random();

    log::debug!(
        "Negotiating ephemeral peer with the ingress relay {} (pq={}, daita={})",
        config.relays.ingress().endpoint,
        negotiate.post_quantum,
        negotiate.daita
    );
    let ingress_peer =
        negotiate_with_ingress(config, negotiate, &ingress_transport, &ephemeral_key).await?;
    log::debug!(
        "Negotiated ephemeral peer with the ingress relay (psk={}, daita={})",
        ingress_peer.psk.is_some(),
        ingress_peer.daita.is_some()
    );

    let (exit_private_key, exit_psk) = match &config.relays {
        Relays::Singlehop(_) => (None, None),
        Relays::Multihop { entry, exit } => {
            let exit_private_key = config.separate_exit_key.then(PrivateKey::new_from_random);
            log::debug!(
                "Negotiating ephemeral peer with the exit relay {}",
                exit.endpoint
            );
            let exit_peer = negotiate_through_entry(
                config,
                negotiate,
                &ingress_transport,
                entry,
                &ephemeral_key,
                ingress_peer.psk.as_ref(),
                exit,
                exit_private_key.as_ref().unwrap_or(&ephemeral_key),
            )
            .await?;
            log::debug!(
                "Negotiated ephemeral peer with the exit relay (psk={})",
                exit_peer.psk.is_some()
            );
            (exit_private_key, exit_peer.psk)
        }
    };

    Ok(NegotiatedPeers {
        private_key: ephemeral_key,
        exit_private_key,
        ingress_psk: ingress_peer.psk,
        exit_psk,
        daita: ingress_peer.daita,
    })
}

/// Negotiate an ephemeral peer with the ingress relay, through a device that uses the private key
/// of this device.
async fn negotiate_with_ingress<F: UdpTransportFactory>(
    config: &NegotiationConfig,
    negotiate: Negotiables,
    ingress_transport: impl Fn(&PublicKey) -> F,
    ephemeral_key: &PrivateKey,
) -> Result<EphemeralPeer, NegotiationError> {
    let (net, net_recv, net_send, _net_guard) = userspace_net(config);

    let peer = ingress_peer(config, config.relays.ingress()).with_allowed_ip(config.allowed_ip());
    let device = DeviceBuilder::new()
        .with_udp(ingress_transport(&config.private_key.public_key()))
        .with_ip_pair(net_send, net_recv)
        .with_private_key(static_secret(&config.private_key))
        .with_peer(peer)
        .build()
        .await
        .map_err(NegotiationError::Device)?;

    let result = tokio::select! {
        result = request_ephemeral_peer_through(net, config, ephemeral_key, negotiate) => result,
        error = fail_without_handshake(&device, config.handshake_timeout) => Err(error),
    };
    device.stop().await;
    result
}

/// Negotiate an ephemeral peer with the exit relay, through the entry relay.
///
/// The entry device uses `entry_key` and `entry_psk`, which were negotiated with the entry relay.
/// The exit device uses the private key of this device, and requests an ephemeral peer for
/// `exit_key`.
#[expect(clippy::too_many_arguments)]
async fn negotiate_through_entry<F: UdpTransportFactory>(
    config: &NegotiationConfig,
    negotiate: Negotiables,
    ingress_transport: impl Fn(&PublicKey) -> F,
    entry: &Relay,
    entry_key: &PrivateKey,
    entry_psk: Option<&PresharedKey>,
    exit: &Relay,
    exit_key: &PrivateKey,
) -> Result<EphemeralPeer, NegotiationError> {
    let (net, net_recv, net_send, _net_guard) = userspace_net(config);
    let (entry_ip_send, entry_ip_recv, exit_udp) = new_udp_tun_channel(
        MULTIHOP_CHANNEL_CAPACITY,
        config.tunnel_ipv4,
        // The exit relay is always reached over IPv4, so the exit device never sends from an IPv6
        // address.
        Ipv6Addr::UNSPECIFIED,
        entry_mtu(exit.endpoint),
    );

    let exit_peer = Peer::new(exit.pubkey())
        .with_endpoint(exit.endpoint)
        .with_allowed_ip(config.allowed_ip());
    let exit_device = DeviceBuilder::new()
        .with_udp(exit_udp)
        .with_ip_pair(net_send, net_recv)
        .with_private_key(static_secret(&config.private_key))
        .with_peer(exit_peer)
        .build()
        .await
        .map_err(NegotiationError::Device)?;

    let mut entry_peer =
        ingress_peer(config, entry).with_allowed_ip(IpNetwork::from(exit.endpoint.ip()));
    if let Some(psk) = entry_psk {
        entry_peer = entry_peer.with_preshared_key(*psk.as_bytes());
    }
    let entry_device = match DeviceBuilder::new()
        .with_udp(ingress_transport(&entry_key.public_key()))
        .with_ip_pair(entry_ip_send, entry_ip_recv)
        .with_private_key(static_secret(entry_key))
        .with_peer(entry_peer)
        .build()
        .await
    {
        Ok(device) => device,
        Err(error) => {
            exit_device.stop().await;
            return Err(NegotiationError::Device(error));
        }
    };

    // DAITA is only used with the ingress relay.
    let negotiate = Negotiables {
        daita: false,
        ..negotiate
    };
    // The exit device reaches the exit relay through the entry relay, so it cannot complete a
    // handshake unless both relays are reachable.
    let result = tokio::select! {
        result = request_ephemeral_peer_through(net, config, exit_key, negotiate) => result,
        error = fail_without_handshake(&exit_device, config.handshake_timeout) => Err(error),
    };
    entry_device.stop().await;
    exit_device.stop().await;
    result
}

/// Return [`NegotiationError::Timeout`] if `device` has not completed a handshake with its peer
/// within `timeout`. Never returns otherwise.
// TODO: consider upstreaming a function that forces a handshake to gotatun.
async fn fail_without_handshake(
    device: &Device<impl DeviceTransports>,
    timeout: Duration,
) -> NegotiationError {
    let handshake = async {
        loop {
            let peers = device.read(async |device| device.peers().await).await;
            if peers.iter().all(|peer| peer.stats.last_handshake.is_some()) {
                log::debug!("Handshake with relays complete");
                return;
            }
            tokio::time::sleep(HANDSHAKE_POLL_INTERVAL).await;
        }
    };
    if tokio::time::timeout(timeout, handshake).await.is_ok() {
        std::future::pending::<()>().await;
    }
    log::debug!("No handshake with the relay within {timeout:?}");
    NegotiationError::Timeout
}

/// Request an ephemeral peer from the config service that is reached through `net`.
async fn request_ephemeral_peer_through(
    net: SmoltcpHandle,
    config: &NegotiationConfig,
    ephemeral_key: &PrivateKey,
    negotiate: Negotiables,
) -> Result<EphemeralPeer, NegotiationError> {
    let config_service = SocketAddr::new(IpAddr::V4(config.config_service_ip), CONFIG_SERVICE_PORT);
    let exchange = async {
        let stream = net
            .set_timeout(config.tcp_timeout)
            .log_stats()
            .tcp_connect(config_service)
            .await
            .map_err(Error::TcpSocketError)?;
        request_ephemeral_peer_over_stream(
            stream,
            config.private_key.public_key(),
            ephemeral_key.public_key(),
            negotiate.post_quantum,
            negotiate.daita,
        )
        .await
    };

    tokio::time::timeout(config.timeout, exchange)
        .await
        .map_err(|_elapsed| NegotiationError::Timeout)?
        .map_err(NegotiationError::Exchange)
}

/// A userspace network stack that uses the tunnel address of this device.
fn userspace_net(
    config: &NegotiationConfig,
) -> (
    SmoltcpHandle,
    SmoltcpIpRecv,
    SmoltcpIpSend,
    SmoltcpNetworkGuard,
) {
    smoltcp_network(SmoltcpNetworkConfig {
        ipv4_addr: config.tunnel_ipv4,
        ipv6_addr: None,
        mtu: USERSPACE_NET_MTU,
    })
}

/// A peer for `relay`, which is the ingress relay.
fn ingress_peer(config: &NegotiationConfig, relay: &Relay) -> Peer {
    let peer = Peer::new(relay.pubkey()).with_endpoint(relay.endpoint);
    match &config.ingress_timer_params {
        Some(timer_params) => peer.dangerously_with_timer_params(timer_params.clone()),
        None => peer,
    }
}

fn static_secret(private_key: &PrivateKey) -> StaticSecret {
    StaticSecret::from(private_key.to_bytes())
}

/// MTU of the entry device, which carries the packets that the exit device sends to `exit_endpoint`.
fn entry_mtu(exit_endpoint: SocketAddr) -> MtuWatcher {
    let overhead = match exit_endpoint {
        SocketAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
        SocketAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
    };
    MtuWatcher::new(USERSPACE_NET_MTU)
        .increase(overhead as u16)
        .expect("the userspace network MTU leaves room for the multihop overhead")
}
