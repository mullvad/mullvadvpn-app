use jnix::jni::{objects::GlobalRef, JavaVM};
use jnix::{IntoJava, FromJava};
use std::sync::Arc;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network, IpNetworkError};
use std::net::IpAddr;

#[derive(Clone)]
pub struct AndroidContext {
    pub jvm: Arc<JavaVM>,
    pub vpn_service: GlobalRef,
}


/// A Java-compatible variant of [IpNetwork]
#[derive(Clone, Debug, Eq, PartialEq, IntoJava, FromJava)]
#[jnix(package = "net.mullvad.talpid.model")]
pub struct InetNetwork {
    pub address: IpAddr,
    pub prefix: i16,
}

impl From<IpNetwork> for InetNetwork {
    fn from(ip_network: IpNetwork) -> Self {
        InetNetwork {
            address: ip_network.ip(),
            prefix: ip_network.prefix() as i16,
        }
    }
}

impl TryFrom<InetNetwork> for IpNetwork {
    type Error = IpNetworkError;
    fn try_from(inet_network: InetNetwork) -> Result<Self, Self::Error> {
        Ok(match inet_network.address {
            IpAddr::V4(addr) => IpNetwork::V4(Ipv4Network::new(addr, inet_network.prefix as u8)?),
            IpAddr::V6(addr) => IpNetwork::V6(Ipv6Network::new(addr, inet_network.prefix as u8)?),
        })
    }
}
