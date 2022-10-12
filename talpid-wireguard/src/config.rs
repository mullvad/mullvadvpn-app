use std::{
    borrow::Cow,
    ffi::CString,
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{obfuscation::ObfuscatorConfig, wireguard, GenericTunnelOptions};

/// Config required to set up a single WireGuard tunnel
#[derive(Clone)]
pub struct Config {
    /// Contains tunnel endpoint specific config
    pub tunnel: wireguard::TunnelConfig,
    /// List of peer configurations
    pub peers: Vec<wireguard::PeerConfig>,
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
    /// Temporary switch for wireguard-nt
    #[cfg(target_os = "windows")]
    pub use_wireguard_nt: bool,
    /// Obfuscator config to be used for reaching the relay.
    pub obfuscator_config: Option<ObfuscatorConfig>,
}

#[cfg(not(target_os = "android"))]
const DEFAULT_MTU: u16 = 1380;

/// Set the MTU to the lowest possible whilst still allowing for IPv6 to help with wireless
/// carriers that do a lot of encapsulation.
#[cfg(target_os = "android")]
const DEFAULT_MTU: u16 = 1280;

/// Configuration errors
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Supplied parameters don't contain a valid tunnel IP
    #[error(display = "No valid tunnel IP")]
    InvalidTunnelIpError,

    /// Peer has no valid IPs
    #[error(display = "Supplied peer has no valid IPs")]
    InvalidPeerIpError,

    /// Parameters don't contain any peers
    #[error(display = "No peers supplied")]
    NoPeersSuppliedError,
}

impl Config {
    /// Constructs a Config from parameters
    pub fn from_parameters(params: &wireguard::TunnelParameters) -> Result<Config, Error> {
        let tunnel = params.connection.tunnel.clone();
        let mut peers = vec![params.connection.peer.clone()];
        if let Some(exit_peer) = &params.connection.exit_peer {
            peers.push(exit_peer.clone());
        }
        Self::new(
            tunnel,
            peers,
            &params.connection,
            &params.options,
            &params.generic_options,
            params.obfuscation.clone(),
        )
    }

    /// Constructs a new Config struct
    pub fn new(
        mut tunnel: wireguard::TunnelConfig,
        mut peers: Vec<wireguard::PeerConfig>,
        connection_config: &wireguard::ConnectionConfig,
        wg_options: &wireguard::TunnelOptions,
        generic_options: &GenericTunnelOptions,
        obfuscator_config: Option<ObfuscatorConfig>,
    ) -> Result<Config, Error> {
        if peers.is_empty() {
            return Err(Error::NoPeersSuppliedError);
        }
        let mtu = wg_options.mtu.unwrap_or(DEFAULT_MTU);
        for peer in &mut peers {
            peer.allowed_ips = peer
                .allowed_ips
                .iter()
                .cloned()
                .filter(|ip| ip.is_ipv4() || generic_options.enable_ipv6)
                .collect();
            if peer.allowed_ips.is_empty() {
                return Err(Error::InvalidPeerIpError);
            }
        }

        if tunnel.addresses.is_empty() {
            return Err(Error::InvalidTunnelIpError);
        }
        tunnel
            .addresses
            .retain(|ip| ip.is_ipv4() || generic_options.enable_ipv6);

        let ipv6_gateway = if generic_options.enable_ipv6 {
            connection_config.ipv6_gateway
        } else {
            None
        };

        Ok(Config {
            tunnel,
            peers,
            ipv4_gateway: connection_config.ipv4_gateway,
            ipv6_gateway,
            mtu,
            #[cfg(target_os = "linux")]
            fwmark: connection_config.fwmark,
            #[cfg(target_os = "linux")]
            enable_ipv6: generic_options.enable_ipv6,
            #[cfg(target_os = "windows")]
            use_wireguard_nt: wg_options.use_wireguard_nt,
            obfuscator_config,
        })
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

        for peer in &self.peers {
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
