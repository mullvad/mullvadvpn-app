use ipnetwork::IpNetwork;
use std::net::{IpAddr, SocketAddr};
use talpid_types::net::{TunnelOptions, WireguardEndpointData};

pub struct PeerConfig {
    pub public_key: [u8; 32],
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
}

pub struct TunnelConfig {
    pub private_key: [u8; 32],
    pub addresses: Vec<IpAddr>,
    pub fwmark: i32,
    pub mtu: u16,
    pub peers: Vec<PeerConfig>,
}

pub struct Config {
    pub interface: TunnelConfig,
    pub pingable_address: IpAddr,
    pub gateway: IpAddr,
    pub preferred_name: Option<String>,
}

fn all_of_the_internet() -> Vec<IpNetwork> {
    vec![
        "::0/0".parse().expect("Failed to parse ipv6 network"),
        "0.0.0.0/0".parse().expect("Failed to parse ipv4 network"),
    ]
}

const DEFAULT_MTU: u16 = 1420;
const DEFAULT_FWMARK: i32 = 787878;

impl Config {
    pub fn from_data(ip: IpAddr, data: WireguardEndpointData, options: &TunnelOptions) -> Config {
        let peer = PeerConfig {
            public_key: data.peer_key,
            allowed_ips: all_of_the_internet(),
            endpoint: SocketAddr::new(ip, data.port),
        };

        let tunnel_config = TunnelConfig {
            private_key: data.client_key,
            addresses: data.addresses,
            mtu: options.wireguard.mtu.unwrap_or(DEFAULT_MTU),
            fwmark: options.wireguard.fwmark.unwrap_or(DEFAULT_FWMARK),
            peers: vec![peer],
        };

        Config {
            interface: tunnel_config,
            pingable_address: data.gateway,
            gateway: data.gateway,
            preferred_name: Some("talpid".to_string()),
        }
    }

    // should probably take a flag that alters between additive and overwriting conf
    pub fn get_wg_config(&self) -> Vec<u8> {
        // the order of insertion matters, public key entry denotes a new peer entry
        let mut wg_conf = WgConfigBuffer::new();
        wg_conf
            .add("private_key", ConfValue::Bytes(&self.interface.private_key))
            .add(
                "fwmark",
                ConfValue::String(&self.interface.fwmark.to_string()),
            )
            .add("listen_port", "0")
            .add("replace_peers", "true");

        for peer in &self.interface.peers {
            wg_conf
                .add("public_key", peer.public_key.as_ref())
                .add("replace_allowed_ips", "true");
            for addr in &peer.allowed_ips {
                wg_conf.add("allowed_ip", addr.to_string().as_str());
            }
        }

        wg_conf.to_config()
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
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            ConfValue::String(s) => s.as_bytes().into(),
            ConfValue::Bytes(bytes) => hex::encode(bytes).as_bytes().into(),
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
        self.buf.extend(value.into().as_bytes());
        self.buf.extend(b"\n");
        self
    }

    pub fn to_config(mut self) -> Vec<u8> {
        self.buf.push(b'\n');
        self.buf
    }
}
