use std::str::FromStr;
use talpid_types::net::TransportProtocol;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct RelayConstraints {
    pub host: Option<String>,
    pub tunnel: TunnelConstraints,
}

impl RelayConstraints {
    pub fn merge(&mut self, update: RelayConstraintsUpdate) -> Self {

        RelayConstraints {
            host: update.host.unwrap_or_else(|| self.host.clone()),
            tunnel: self.tunnel.merge(update.tunnel),
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub enum TunnelConstraints {
    OpenVpn(OpenVpnConstraints),
}

impl TunnelConstraints {
    pub fn merge(&mut self, update: TunnelConstraintsUpdate) -> Self {
        match *self {
            TunnelConstraints::OpenVpn(ref mut current) => match update {
                TunnelConstraintsUpdate::OpenVpn(openvpn_update) => TunnelConstraints::OpenVpn(current.merge(openvpn_update)),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct OpenVpnConstraints {
    pub port: Port,
    pub protocol: TransportProtocol,
}

impl OpenVpnConstraints {
    pub fn merge(&mut self, update: OpenVpnConstraintsUpdate) -> Self {
        OpenVpnConstraints {
            port: update.port.unwrap_or(self.port),
            protocol: update.protocol.unwrap_or(self.protocol),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct RelayConstraintsUpdate {
    pub host: Option<Option<String>>,
    pub tunnel: TunnelConstraintsUpdate,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TunnelConstraintsUpdate {
    OpenVpn(OpenVpnConstraintsUpdate),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenVpnConstraintsUpdate {
    pub port: Option<Port>,
    pub protocol: Option<TransportProtocol>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Port {
    Any,
    Port(u16),
}

impl FromStr for Port {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = u16::from_str_radix(s, 10);
        match res {
            Ok(num) => Ok(Port::Port(num)),
            Err(_) => if s.to_lowercase() == "any" {
                Ok(Port::Any)
            } else {
                Err("not 'any' or a short".to_owned())
            },
        }
    }
}
