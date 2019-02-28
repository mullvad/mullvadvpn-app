use crate::net::{Endpoint, GenericTunnelOptions, TransportProtocol, TunnelEndpoint, TunnelType};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use rand::RngCore;


#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
/// Wireguard tunnel parameters
pub struct TunnelParameters {
    pub connection: ConnectionConfig,
    pub options: TunnelOptions,
    pub generic_options: GenericTunnelOptions,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub tunnel: TunnelConfig,
    pub peer: PeerConfig,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Option<Ipv6Addr>,
}

impl ConnectionConfig {
    pub fn get_tunnel_endpoint(&self) -> TunnelEndpoint {
        let host = self.peer.endpoint;
        TunnelEndpoint {
            tunnel_type: TunnelType::Wireguard,
            endpoint: Endpoint {
                address: host,
                protocol: TransportProtocol::Udp,
            },
        }
    }
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug, Hash)]
pub struct PeerConfig {
    pub public_key: PublicKey,
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub struct TunnelConfig {
    pub private_key: PrivateKey,
    pub addresses: Vec<IpAddr>,
}

/// Wireguard tunnel options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunnelOptions {
    /// MTU for the wireguard tunnel
    pub mtu: Option<u16>,
    /// firewall mark
    #[cfg(target_os = "linux")]
    pub fwmark: i32,
}

/// Wireguard x25519 private key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrivateKey([u8; 32]);

impl PrivateKey {
    /// Get private key as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Normalizing a private key as per the specification - https://cr.yp.to/ecdh.html
    fn normalize_key(bytes: &mut [u8; 32]) {
        bytes[0] &= 248;
        bytes[31] &= 127;
        bytes[31] |= 64;
    }

    pub fn new_from_random() -> Result<Self, rand::Error> {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng::new()?.fill_bytes(&mut bytes);
        Ok(Self::from(bytes))
    }

    /// Generate public key from private key
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(x25519_dalek::x25519(
            self.0,
            x25519_dalek::X25519_BASEPOINT_BYTES,
        ))
    }
}

impl From<[u8; 32]> for PrivateKey {
    fn from(mut private_key: [u8; 32]) -> PrivateKey {
        Self::normalize_key(&mut private_key);
        PrivateKey(private_key)
    }
}

impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_key(&self.0, serializer)
    }
}


impl<'de> Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_key(deserializer)
    }
}

/// Wireguard x25519 public key
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Get the public key as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}


impl From<[u8; 32]> for PublicKey {
    fn from(public_key: [u8; 32]) -> PublicKey {
        PublicKey(public_key)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_key(&self.0, serializer)
    }
}


impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_key(deserializer)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &base64::encode(&self.0))
    }
}

fn serialize_key<S>(key: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(&key))
}

fn deserialize_key<'de, D, K>(deserializer: D) -> Result<K, D::Error>
where
    D: Deserializer<'de>,
    K: From<[u8; 32]>,
{
    use serde::de::Error;

    String::deserialize(deserializer)
        .and_then(|string| base64::decode(&string).map_err(|err| Error::custom(err.to_string())))
        .and_then(|buffer| {
            let mut key = [0u8; 32];
            if buffer.len() != 32 {
                return Err(Error::custom(format!(
                    "Key has unexpected length: {}",
                    buffer.len()
                )));
            }
            key.copy_from_slice(&buffer);
            Ok(From::from(key))
        })
}
