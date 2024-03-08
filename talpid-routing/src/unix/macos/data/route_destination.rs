use super::{Error, RouteMessage};
use ipnetwork::IpNetwork;
use std::net::IpAddr;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct RouteDestination {
    pub network: IpNetwork,
    pub interface: Option<u16>,
    pub gateway: Option<IpAddr>,
}

impl TryFrom<&RouteMessage> for RouteDestination {
    type Error = Error;

    fn try_from(msg: &RouteMessage) -> std::result::Result<Self, Self::Error> {
        let network = msg.destination_ip()?;
        let interface = msg.ifscope();
        let gateway = msg.gateway_ip();
        Ok(Self {
            network,
            interface,
            gateway,
        })
    }
}
