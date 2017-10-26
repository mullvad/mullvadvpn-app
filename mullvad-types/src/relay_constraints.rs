use std::str::FromStr;
use talpid_types::net::TransportProtocol;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayConstraints {
    pub host: Option<String>,
    pub tunnel: TunnelConstraints,
}

impl RelayConstraints {
    pub fn update(&mut self, update: RelayConstraintsUpdate) -> bool {
        let mut updated = false;

        if let Some(new_host) = update.host {
            self.host = new_host;
            updated = true;
        }

        updated || self.tunnel.update(update.tunnel)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum TunnelConstraints {
    OpenVpn(OpenVpnConstraints),
}

impl TunnelConstraints {
    pub fn update(&mut self, update: TunnelConstraintsUpdate) -> bool {
        match *self {
            TunnelConstraints::OpenVpn(ref mut current) => match update {
                TunnelConstraintsUpdate::OpenVpn(openvpn_update) => current.update(openvpn_update),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OpenVpnConstraints {
    pub port: Port,
    pub protocol: TransportProtocol,
}

impl OpenVpnConstraints {
    pub fn update(&mut self, update: OpenVpnConstraintsUpdate) -> bool {
        let mut updated = false;

        if let Some(new_port) = update.port {
            self.port = new_port;
            updated = true;
        }
        if let Some(new_protocol) = update.protocol {
            self.protocol = new_protocol;
            updated = true;
        }

        updated
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
