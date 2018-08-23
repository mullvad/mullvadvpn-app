use location::{CityCode, CountryCode, Location};

use std::net::Ipv4Addr;

use talpid_types::net::{OpenVpnEndpointData, WireguardEndpointData};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayList {
    pub countries: Vec<RelayListCountry>,
}

impl RelayList {
    pub fn empty() -> Self {
        Self {
            countries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCountry {
    pub name: String,
    pub code: CountryCode,
    pub cities: Vec<RelayListCity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCity {
    pub name: String,
    pub code: CityCode,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_deserializing)]
    pub has_active_relays: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub relays: Vec<Relay>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relay {
    pub hostname: String,
    pub ipv4_addr_in: Ipv4Addr,
    pub ipv4_addr_exit: Ipv4Addr,
    pub include_in_country: bool,
    pub weight: u64,
    #[serde(skip_serializing_if = "RelayTunnels::is_empty", default)]
    pub tunnels: RelayTunnels,
    #[serde(skip)]
    pub location: Option<Location>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RelayTunnels {
    pub openvpn: Vec<OpenVpnEndpointData>,
    pub wireguard: Vec<WireguardEndpointData>,
}

impl RelayTunnels {
    pub fn is_empty(&self) -> bool {
        self.openvpn.is_empty() && self.wireguard.is_empty()
    }

    pub fn clear(&mut self) {
        self.openvpn.clear();
        self.wireguard.clear();
    }
}
