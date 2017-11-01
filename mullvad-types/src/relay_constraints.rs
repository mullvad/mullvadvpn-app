use std::fmt;
use talpid_types::net::TransportProtocol;


#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Constraint<T: fmt::Debug + Clone + Eq + PartialEq> {
    Any,
    Only(T),
}

impl<T: fmt::Debug + Clone + Eq + PartialEq> Default for Constraint<T> {
    fn default() -> Self {
        Constraint::Any
    }
}

impl<T: Copy + fmt::Debug + Clone + Eq + PartialEq> Copy for Constraint<T> {}


#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayConstraints {
    pub host: Constraint<String>,
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


#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelConstraints {
    #[serde(rename = "openvpn")] OpenVpn(OpenVpnConstraints),
}

impl Default for TunnelConstraints {
    fn default() -> Self {
        TunnelConstraints::OpenVpn(OpenVpnConstraints::default())
    }
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenVpnConstraints {
    pub port: Constraint<u16>,
    pub protocol: Constraint<TransportProtocol>,
}

impl OpenVpnConstraints {
    pub fn merge(&mut self, update: OpenVpnConstraintsUpdate) -> Self {
        OpenVpnConstraints {
            port: update.port.unwrap_or_else(|| self.port.clone()),
            protocol: update.protocol.unwrap_or(self.protocol),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayConstraintsUpdate {
    pub host: Option<Constraint<String>>,
    pub tunnel: TunnelConstraintsUpdate,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelConstraintsUpdate {
    #[serde(rename = "openvpn")] OpenVpn(OpenVpnConstraintsUpdate),
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenVpnConstraintsUpdate {
    pub port: Option<Constraint<u16>>,
    pub protocol: Option<Constraint<TransportProtocol>>,
}
