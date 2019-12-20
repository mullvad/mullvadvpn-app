use std::{
    borrow::Cow,
    ffi::CString,
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{wireguard, GenericTunnelOptions};

pub struct Config {
    pub tunnel: wireguard::TunnelConfig,
    pub peers: Vec<wireguard::PeerConfig>,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Option<Ipv6Addr>,
    pub mtu: u16,
}

/// Smallest MTU that supports IPv6
const SMALLEST_IPV6_MTU: u16 = 1380;
const DEFAULT_MTU: u16 = SMALLEST_IPV6_MTU;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "No valid tunnel IP")]
    InvalidTunnelIpError,

    #[error(display = "Supplied peer has no valid IPs")]
    InvalidPeerIpError,

    #[error(display = "No peers supplied")]
    NoPeersSuppliedError,
}

impl Config {
    pub fn from_parameters(params: &wireguard::TunnelParameters) -> Result<Config, Error> {
        let tunnel = params.connection.tunnel.clone();
        let peer = vec![params.connection.peer.clone()];
        Self::new(
            tunnel,
            peer,
            &params.connection,
            &params.options,
            &params.generic_options,
        )
    }

    pub fn new(
        mut tunnel: wireguard::TunnelConfig,
        mut peers: Vec<wireguard::PeerConfig>,
        connection_config: &wireguard::ConnectionConfig,
        wg_options: &wireguard::TunnelOptions,
        generic_options: &GenericTunnelOptions,
    ) -> Result<Config, Error> {
        if peers.is_empty() {
            return Err(Error::NoPeersSuppliedError);
        }
        let mtu = wg_options.mtu.unwrap_or(DEFAULT_MTU);
        let is_ipv6_enabled = mtu >= SMALLEST_IPV6_MTU && generic_options.enable_ipv6;

        for peer in &mut peers {
            peer.allowed_ips = peer
                .allowed_ips
                .iter()
                .cloned()
                .filter(|ip| ip.is_ipv4() || is_ipv6_enabled)
                .collect();
            if peer.allowed_ips.is_empty() {
                return Err(Error::InvalidPeerIpError);
            }
        }

        tunnel.addresses = tunnel
            .addresses
            .into_iter()
            .filter(|ip| ip.is_ipv4() || is_ipv6_enabled)
            .collect();
        if tunnel.addresses.is_empty() {
            return Err(Error::InvalidTunnelIpError);
        }

        Ok(Config {
            tunnel,
            peers,
            ipv4_gateway: connection_config.ipv4_gateway,
            // Only set the v6 gateway if setting a v6 gateway makes sense
            ipv6_gateway: if is_ipv6_enabled {
                connection_config.ipv6_gateway
            } else {
                None
            },
            mtu,
        })
    }

    // should probably take a flag that alters between additive and overwriting conf
    pub fn to_userspace_format(&self) -> CString {
        // the order of insertion matters, public key entry denotes a new peer entry
        let mut wg_conf = WgConfigBuffer::new();
        wg_conf
            .add("private_key", self.tunnel.private_key.as_bytes().as_ref())
            .add("listen_port", "0");

        wg_conf.add("replace_peers", "true");

        for peer in &self.peers {
            wg_conf
                .add("public_key", peer.public_key.as_bytes().as_ref())
                .add("endpoint", peer.endpoint.to_string().as_str())
                .add("replace_allowed_ips", "true");
            for addr in &peer.allowed_ips {
                wg_conf.add("allowed_ip", addr.to_string().as_str());
            }
        }

        let bytes = wg_conf.into_config();
        CString::new(bytes).expect("null bytes inside config")
    }
}

pub enum ConfValue<'a> {
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

pub struct WgConfigBuffer {
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

/// A config that is currently set
pub struct TunnelConfig {}
