use talpid_types::net::TransportProtocol;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct RelayConstraints {
    pub host: HostConstraint,
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
#[serde(rename_all = "snake_case")]
pub enum TunnelConstraints {
    #[serde(rename = "openvpn")] OpenVpn(OpenVpnConstraints),
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
#[serde(rename_all = "snake_case")]
pub struct OpenVpnConstraints {
    pub port: PortConstraint,
    pub protocol: ProtocolConstraint,
}

impl OpenVpnConstraints {
    pub fn merge(&mut self, update: OpenVpnConstraintsUpdate) -> Self {
        OpenVpnConstraints {
            port: update.port.unwrap_or_else(|| self.port.clone()),
            protocol: update.protocol.unwrap_or_else(|| self.protocol.clone()),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayConstraintsUpdate {
    pub host: Option<HostConstraint>,
    pub tunnel: TunnelConstraintsUpdate,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelConstraintsUpdate {
    #[serde(rename = "openvpn")] OpenVpn(OpenVpnConstraintsUpdate),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenVpnConstraintsUpdate {
    pub port: Option<PortConstraint>,
    pub protocol: Option<ProtocolConstraint>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HostConstraint {
    Any,
    Host(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortConstraint {
    Any,
    Port(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolConstraint {
    Any,
    Protocol(TransportProtocol),
}
