use ipnetwork::{IpNetwork, IpNetworkError, Ipv4Network, Ipv6Network};
use jnix::jni::{objects::GlobalRef, JavaVM};
use jnix::{FromJava, IntoJava};
use std::net::IpAddr;
use std::sync::Arc;

/// What Java calls an [IpAddr]
pub type InetAddress = IpAddr;

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

#[derive(Clone, Debug, Eq, PartialEq, IntoJava, FromJava)]
#[jnix(package = "net.mullvad.talpid.model")]
pub struct RouteInfo {
    pub destination: InetNetwork,
    pub gateway: Option<InetAddress>,
    pub interface: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoJava, FromJava)]
#[jnix(package = "net.mullvad.talpid.model")]
pub struct NetworkState {
    pub network_handle: i64,
    pub routes: Option<Vec<RouteInfo>>,
    pub dns_servers: Option<Vec<InetAddress>>,
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
