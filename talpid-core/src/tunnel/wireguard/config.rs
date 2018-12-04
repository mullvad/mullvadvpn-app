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
        wg_conf.add(
            "private_key",
            ConfValue::Bytes(&self.interface.private_key.clone()),
        );

        wg_conf.add(
            "fwmark",
            ConfValue::String(&self.interface.fwmark.to_string()),
        );
        wg_conf.add("listen_port", ConfValue::String(&"0"));
        wg_conf.add("replace_peers", ConfValue::String(&"true"));

        for peer in &self.interface.peers {
            wg_conf.add("public_key", ConfValue::Bytes(&peer.public_key));
            wg_conf.add("endpoint", ConfValue::String(&peer.endpoint.to_string()));
            wg_conf.add("replace_allowed_ips", ConfValue::String(&"true"));
            for addr in &peer.allowed_ips {
                wg_conf.add("allowed_ip", ConfValue::String(&addr.to_string()));
            }
        }

        wg_conf.to_config()
    }
}

pub enum ConfValue<'a> {
    String(&'a str),
    Bytes(&'a [u8]),
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

    pub fn add(&mut self, key: &str, value: ConfValue) {
        self.buf.extend(key.as_bytes());
        self.buf.extend(b"=");
        self.buf.extend(value.as_bytes());
        self.buf.extend(b"\n");
    }

    pub fn to_config(mut self) -> Vec<u8> {
        self.buf.push(b'\n');
        self.buf
    }
}
