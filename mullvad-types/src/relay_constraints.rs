use talpid_types::net::TransportProtocol;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct RelayConstraints {
    pub host: Option<HostConstraint>,
    pub tunnel: TunnelConstraints,
}

impl RelayConstraints {
    pub fn merge(&mut self, update: RelayConstraintsUpdate) -> Self {
        RelayConstraints {
            host: update.host.or_else(|| self.host.clone()),
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
                TunnelConstraintsUpdate::OpenVpn(openvpn_update) => {
                    TunnelConstraints::OpenVpn(current.merge(openvpn_update))
                }
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct OpenVpnConstraints {
    pub port: Option<Port>,
    pub protocol: Option<TransportProtocol>,
}

impl OpenVpnConstraints {
    pub fn merge(&mut self, update: OpenVpnConstraintsUpdate) -> Self {
        OpenVpnConstraints {
            port: update.port.or_else(|| self.port.clone()),
            protocol: update.protocol.or(self.protocol),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct RelayConstraintsUpdate {
    pub host: Option<HostConstraint>,
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


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum HostConstraint {
    Any,
    Host(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Port {
    Any,
    Port(u16),
}
