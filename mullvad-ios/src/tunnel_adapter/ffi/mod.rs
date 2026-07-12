//! uniffi FFI for the iOS tunnel adapter.
//!
//! Swift builds a [`GotaTunConfig`] record (with [`GotaTunPeer`] and
//! [`GotaTunObfuscation`]) and constructs a [`GotaTunTunnel`] object, passing a
//! foreign-implemented [`GotaTunCallback`]. All validation/parsing happens in
//! [`GotaTunTunnel::start`]; the object owns the running [`IosTunnelAdapter`] and
//! stops it on drop.

use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use ipnetwork::IpNetwork;

use super::{IosTunnelAdapter, ObfuscationConfig, PeerConfig, TunnelCallbackHandler, TunnelConfig};

/// A WireGuard peer (entry or exit).
#[derive(uniffi::Record)]
pub struct GotaTunPeer {
    /// Peer's WireGuard public key (32 bytes).
    pub public_key: Vec<u8>,
    /// Peer endpoint as "ip:port".
    pub endpoint: String,
}

/// Full tunnel configuration.
#[derive(uniffi::Record)]
pub struct GotaTunConfig {
    /// WireGuard private key (32 bytes).
    pub private_key: Vec<u8>,
    /// Tunnel interface IPv4 address (e.g. "10.64.0.2").
    pub ipv4_address: String,
    /// Tunnel interface IPv6 address.
    pub ipv6_address: String,
    /// Tunnel MTU.
    pub mtu: u16,
    /// Exit peer (always present).
    pub exit_peer: GotaTunPeer,
    /// Entry peer for multihop, or `None` for singlehop.
    pub entry_peer: Option<GotaTunPeer>,
    /// Gateway IPv4 address used for connectivity pings (e.g. "10.64.0.1").
    pub ipv4_gateway: String,
    /// How long to wait for the tunnel to establish connectivity (seconds).
    pub establish_timeout_secs: u32,
    /// Enable post-quantum key exchange.
    pub enable_pq: bool,
    /// Enable DAITA.
    pub enable_daita: bool,
    /// Obfuscation method for the ingress relay.
    pub obfuscation: GotaTunObfuscation,
}

/// Obfuscation method applied to the ingress relay connection.
#[derive(uniffi::Enum)]
pub enum GotaTunObfuscation {
    Off,
    UdpOverTcp,
    Shadowsocks,
    Quic {
        hostname: String,
        token: String,
    },
    Lwo {
        client_public_key: Vec<u8>,
        server_public_key: Vec<u8>,
    },
}

/// Error returned when starting a tunnel.
#[derive(Debug, uniffi::Error)]
pub enum GotaTunFfiError {
    /// A field in the config was malformed (bad key length, unparseable address, ...).
    InvalidConfig(String),
    /// An internal failure (e.g. the async runtime was unavailable).
    Internal(String),
}

impl std::fmt::Display for GotaTunFfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GotaTunFfiError::InvalidConfig(msg) => write!(f, "invalid config: {msg}"),
            GotaTunFfiError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for GotaTunFfiError {}

/// Callbacks from the tunnel adapter to Swift. Implemented on the Swift side and
/// invoked from Rust-owned threads. Each method fires at most once per adapter,
/// except `on_timeout`, which may fire after `on_connected` if connectivity drops.
#[uniffi::export(callback_interface)]
pub trait GotaTunCallback: Send + Sync + 'static {
    /// The tunnel is connected and traffic flows.
    fn on_connected(&self);
    /// The pinger timed out.
    fn on_timeout(&self);
    /// A fatal error occurred.
    fn on_error(&self, message: String);
}

/// Bridges the foreign [`GotaTunCallback`] to the internal [`TunnelCallbackHandler`].
struct CallbackBridge(Box<dyn GotaTunCallback>);

impl TunnelCallbackHandler for CallbackBridge {
    fn on_connected(&self) {
        self.0.on_connected();
    }
    fn on_timeout(&self) {
        self.0.on_timeout();
    }
    fn on_error(&self, message: String) {
        self.0.on_error(message);
    }
}

/// A running GotaTun tunnel. Dropping it stops the tunnel (via
/// [`IosTunnelAdapter`]'s `Drop`); `stop` is exposed for deterministic teardown.
#[derive(uniffi::Object)]
pub struct GotaTunTunnel {
    adapter: IosTunnelAdapter,
}

#[uniffi::export]
impl GotaTunTunnel {
    /// Start a tunnel with the given TUN file descriptor, config, and callbacks.
    ///
    /// Validates and parses the config, then spawns the adapter. Exactly one of
    /// `on_connected`/`on_timeout`/`on_error` will be invoked on `callback`.
    #[uniffi::constructor]
    pub fn start(
        tun_fd: i32,
        config: GotaTunConfig,
        callback: Box<dyn GotaTunCallback>,
    ) -> Result<Arc<Self>, GotaTunFfiError> {
        let tunnel_config = build_tunnel_config(tun_fd, config)?;
        let runtime = crate::mullvad_ios_runtime().map_err(GotaTunFfiError::Internal)?;
        let handler: Arc<dyn TunnelCallbackHandler> = Arc::new(CallbackBridge(callback));
        let adapter = IosTunnelAdapter::start(runtime, tunnel_config, handler);
        Ok(Arc::new(Self { adapter }))
    }

    /// Stop and tear down the tunnel. Safe to call multiple times.
    pub fn stop(&self) {
        self.adapter.stop();
    }

    /// Recycle UDP sockets after a network path change.
    pub fn recycle_udp_sockets(&self) {
        self.adapter.recycle_udp_sockets();
    }

    /// Suspend the tunnel (device sleep).
    pub fn suspend(&self) {
        self.adapter.suspend();
    }

    /// Wake the tunnel from suspension.
    pub fn wake(&self) {
        self.adapter.wake();
    }
}

fn key32(bytes: &[u8], what: &str) -> Result<[u8; 32], GotaTunFfiError> {
    bytes.try_into().map_err(|_| {
        GotaTunFfiError::InvalidConfig(format!("{what} must be 32 bytes, got {}", bytes.len()))
    })
}

fn parse<T>(value: &str, what: &str) -> Result<T, GotaTunFfiError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .parse()
        .map_err(|e| GotaTunFfiError::InvalidConfig(format!("invalid {what} '{value}': {e}")))
}

/// The full IPv4 + IPv6 internet, used as the exit peer's allowed IPs.
fn catch_all_ips() -> Vec<IpNetwork> {
    vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()]
}

fn build_peer(
    peer: &GotaTunPeer,
    allowed_ips: Vec<IpNetwork>,
) -> Result<PeerConfig, GotaTunFfiError> {
    Ok(PeerConfig {
        public_key: key32(&peer.public_key, "peer public key")?,
        endpoint: parse(&peer.endpoint, "peer endpoint")?,
        allowed_ips,
    })
}

fn build_obfuscation(
    obfuscation: GotaTunObfuscation,
) -> Result<ObfuscationConfig, GotaTunFfiError> {
    Ok(match obfuscation {
        GotaTunObfuscation::Off => ObfuscationConfig::Off,
        GotaTunObfuscation::UdpOverTcp => ObfuscationConfig::UdpOverTcp,
        GotaTunObfuscation::Shadowsocks => ObfuscationConfig::Shadowsocks,
        GotaTunObfuscation::Quic { hostname, token } => ObfuscationConfig::Quic { hostname, token },
        GotaTunObfuscation::Lwo {
            client_public_key,
            server_public_key,
        } => ObfuscationConfig::Lwo {
            client_public_key: key32(&client_public_key, "LWO client public key")?,
            server_public_key: key32(&server_public_key, "LWO server public key")?,
        },
    })
}

fn build_tunnel_config(
    tun_fd: i32,
    config: GotaTunConfig,
) -> Result<TunnelConfig, GotaTunFfiError> {
    // The exit peer carries all user traffic (full internet). In multihop the entry
    // peer only carries the exit relay's encrypted UDP, so its single allowed IP is
    // the exit endpoint's address (a host route).
    let exit_peer = build_peer(&config.exit_peer, catch_all_ips())?;
    let entry_peer = config
        .entry_peer
        .as_ref()
        .map(|peer| build_peer(peer, vec![IpNetwork::from(exit_peer.endpoint.ip())]))
        .transpose()?;

    Ok(TunnelConfig {
        tun_fd,
        private_key: key32(&config.private_key, "private key")?,
        ipv4_addr: parse::<Ipv4Addr>(&config.ipv4_address, "IPv4 address")?,
        ipv6_addr: parse::<Ipv6Addr>(&config.ipv6_address, "IPv6 address")?,
        mtu: config.mtu,
        exit_peer,
        entry_peer,
        ipv4_gateway: parse::<Ipv4Addr>(&config.ipv4_gateway, "IPv4 gateway")?,
        establish_timeout_secs: config.establish_timeout_secs,
        enable_pq: config.enable_pq,
        enable_daita: config.enable_daita,
        obfuscation: build_obfuscation(config.obfuscation)?,
    })
}
