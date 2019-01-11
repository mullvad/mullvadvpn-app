use super::{ErrorKind, Result};
use ipnetwork::IpNetwork;
use std::{
    borrow::Cow,
    ffi::CString,
    net::{IpAddr, SocketAddr},
};
use talpid_types::net::{WgPrivateKey, WgPublicKey, WireguardTunnelParameters};

pub struct Config {
    pub interface: TunnelConfig,
    pub gateway: IpAddr,
}
// Smallest MTU that supports IPv6
const MIN_IPV6_MTU: u16 = 1420;
const DEFAULT_MTU: u16 = MIN_IPV6_MTU;

impl Config {
    pub fn from_data(connection_params: &WireguardTunnelParameters) -> Config {
        let mtu = connection_params.options.mtu.unwrap_or(DEFAULT_MTU);
        let ipv6_enabled = connection_params.generic_options.enable_ipv6 && mtu >= MIN_IPV6_MTU;
        let peer = PeerConfig {
            public_key: connection_params.config.peer_public_key.clone(),
            allowed_ips: all_of_the_internet()
                .into_iter()
                .filter(|ip| ip.is_ipv4() || ipv6_enabled)
                .collect(),
            endpoint: connection_params.config.host,
        };

        let tunnel_params = TunnelConfig {
            private_key: connection_params.config.client_private_key.clone(),
            addresses: connection_params
                .config
                .link_addresses
                .iter()
                .filter(|ip| ip.is_ipv4() || ipv6_enabled)
                .cloned()
                .collect(),
            mtu,
            #[cfg(target_os = "linux")]
            fwmark: connection_params.options.fwmark,
            peers: vec![peer],
        };

        Config {
            interface: tunnel_params,
            gateway: connection_params.config.gateway,
        }
    }

    // should probably take a flag that alters between additive and overwriting conf
    pub fn to_userspace_format(&self) -> CString {
        // the order of insertion matters, public key entry denotes a new peer entry
        let mut wg_conf = WgConfigBuffer::new();
        wg_conf
            .add(
                "private_key",
                self.interface.private_key.as_bytes().as_ref(),
            )
            .add("listen_port", "0");

        #[cfg(target_os = "linux")]
        {
            wg_conf.add("fwmark", self.interface.fwmark.to_string().as_str());
        }

        wg_conf.add("replace_peers", "true");

        for peer in &self.interface.peers {
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

pub struct PeerConfig {
    pub public_key: WgPublicKey,
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
}

pub struct TunnelConfig {
    pub private_key: WgPrivateKey,
    pub addresses: Vec<IpAddr>,
    #[cfg(target_os = "linux")]
    pub fwmark: i32,
    pub mtu: u16,
    pub peers: Vec<PeerConfig>,
}


fn all_of_the_internet() -> Vec<IpNetwork> {
    vec![
        "::0/0".parse().expect("Failed to parse ipv6 network"),
        "0.0.0.0/0".parse().expect("Failed to parse ipv4 network"),
    ]
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
