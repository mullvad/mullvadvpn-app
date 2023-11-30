use std::{
    borrow::Cow,
    ffi::CString,
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{obfuscation::ObfuscatorConfig, wireguard, GenericTunnelOptions};

/// Config required to set up a single WireGuard tunnel
#[derive(Debug, Clone)]
pub struct Config {
    /// Contains tunnel endpoint specific config
    pub tunnel: wireguard::TunnelConfig,
    /// Entry peer
    pub entry_peer: wireguard::PeerConfig,
    /// Multihop exit peer
    pub exit_peer: Option<wireguard::PeerConfig>,
    /// IPv4 gateway
    pub ipv4_gateway: Ipv4Addr,
    /// IPv6 gateway
    pub ipv6_gateway: Option<Ipv6Addr>,
    /// Maximum transmission unit for the tunnel
    pub mtu: u16,
    /// Firewall mark
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
    /// Enable IPv6 routing rules
    #[cfg(target_os = "linux")]
    pub enable_ipv6: bool,
    /// Obfuscator config to be used for reaching the relay.
    pub obfuscator_config: Option<ObfuscatorConfig>,
}

/// Set the MTU to the lowest possible whilst still allowing for IPv6 to help with wireless
/// carriers that do a lot of encapsulation.
const DEFAULT_MTU: u16 = if cfg!(target_os = "android") {
    1280
} else {
    1380
};

/// Configuration errors
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Supplied parameters don't contain a valid tunnel IP
    #[error(display = "No valid tunnel IP")]
    InvalidTunnelIpError,

    /// Peer has no valid IPs
    #[error(display = "Supplied peer has no valid IPs")]
    InvalidPeerIpError,
}

impl Config {
    /// Constructs a Config from parameters
    pub fn from_parameters(params: &wireguard::TunnelParameters) -> Result<Config, Error> {
        Self::new(
            &params.connection,
            &params.options,
            &params.generic_options,
            &params.obfuscation,
        )
    }

    /// Constructs a new Config struct
    fn new(
        connection: &wireguard::ConnectionConfig,
        wg_options: &wireguard::TunnelOptions,
        generic_options: &GenericTunnelOptions,
        obfuscator_config: &Option<ObfuscatorConfig>,
    ) -> Result<Config, Error> {
        let mut tunnel = connection.tunnel.clone();
        let mtu = wg_options.mtu.unwrap_or(DEFAULT_MTU);

        if tunnel.addresses.is_empty() {
            return Err(Error::InvalidTunnelIpError);
        }
        tunnel
            .addresses
            .retain(|ip| ip.is_ipv4() || generic_options.enable_ipv6);

        let ipv6_gateway = connection
            .ipv6_gateway
            .filter(|_opt| generic_options.enable_ipv6);

        let mut config = Config {
            tunnel,
            entry_peer: connection.peer.clone(),
            exit_peer: connection.exit_peer.clone(),
            ipv4_gateway: connection.ipv4_gateway,
            ipv6_gateway,
            mtu,
            #[cfg(target_os = "linux")]
            fwmark: connection.fwmark,
            #[cfg(target_os = "linux")]
            enable_ipv6: generic_options.enable_ipv6,
            obfuscator_config: obfuscator_config.to_owned(),
        };

        for peer in config.peers_mut() {
            peer.allowed_ips
                .retain(|ip| ip.is_ipv4() || generic_options.enable_ipv6);
            if peer.allowed_ips.is_empty() {
                return Err(Error::InvalidPeerIpError);
            }
        }

        Ok(config)
    }

    /// Returns a CString with the appropriate config for WireGuard-go
    // TODO: Consider outputting both overriding and additive configs
    pub fn to_userspace_format(&self) -> CString {
        // the order of insertion matters, public key entry denotes a new peer entry
        let mut wg_conf = WgConfigBuffer::new();
        wg_conf
            .add("private_key", self.tunnel.private_key.to_bytes().as_ref())
            .add("listen_port", "0");

        #[cfg(target_os = "linux")]
        if let Some(fwmark) = &self.fwmark {
            wg_conf.add("fwmark", fwmark.to_string().as_str());
        }

        wg_conf.add("replace_peers", "true");

        for peer in self.peers() {
            wg_conf
                .add("public_key", peer.public_key.as_bytes().as_ref())
                .add("endpoint", peer.endpoint.to_string().as_str())
                .add("replace_allowed_ips", "true");
            if let Some(ref psk) = peer.psk {
                wg_conf.add("preshared_key", psk.as_bytes().as_ref());
            }
            for addr in &peer.allowed_ips {
                wg_conf.add("allowed_ip", addr.to_string().as_str());
            }
        }

        let bytes = wg_conf.into_config();
        CString::new(bytes).expect("null bytes inside config")
    }

    /// Return whether the config connects to an exit peer from another remote peer.
    pub fn is_multihop(&self) -> bool {
        self.exit_peer.is_some()
    }

    /// Return the exit peer. `exit_peer` if it is set, otherwise `entry_peer`.
    pub fn exit_peer_mut(&mut self) -> &mut wireguard::PeerConfig {
        if let Some(ref mut peer) = self.exit_peer {
            return peer;
        }
        &mut self.entry_peer
    }

    /// Return an iterator over all peers.
    pub fn peers(&self) -> impl Iterator<Item = &wireguard::PeerConfig> {
        self.exit_peer
            .as_ref()
            .into_iter()
            .chain(std::iter::once(&self.entry_peer))
    }

    /// Return a mutable iterator over all peers.
    pub fn peers_mut(&mut self) -> impl Iterator<Item = &mut wireguard::PeerConfig> {
        self.exit_peer
            .as_mut()
            .into_iter()
            .chain(std::iter::once(&mut self.entry_peer))
    }
}

enum ConfValue<'a> {
    String(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> From<&'a str> for ConfValue<'a> {
    fn from(s: &'a str) -> ConfValue<'a> {
        ConfValue::String(s)
    }
}

impl<'a> From<&'a [u8]> for ConfValue<'a> {
    fn from(s: &'a [u8]) -> ConfValue<'a> {
        ConfValue::Bytes(s)
    }
}

impl<'a> ConfValue<'a> {
    fn to_bytes(&self) -> Cow<'a, [u8]> {
        match self {
            ConfValue::String(s) => s.as_bytes().into(),
            ConfValue::Bytes(bytes) => Cow::Owned(hex::encode(bytes).into_bytes()),
        }
    }
}

struct WgConfigBuffer {
    buf: Vec<u8>,
}

impl WgConfigBuffer {
    pub fn new() -> WgConfigBuffer {
        WgConfigBuffer { buf: Vec::new() }
    }

    pub fn add<'a, C: Into<ConfValue<'a>> + 'a>(&mut self, key: &str, value: C) -> &mut Self {
        self.buf.extend(key.as_bytes());
        self.buf.extend(b"=");
        self.buf.extend(value.into().to_bytes().as_ref());
        self.buf.extend(b"\n");
        self
    }

    pub fn into_config(mut self) -> Vec<u8> {
        self.buf.push(b'\n');
        self.buf
    }
}
