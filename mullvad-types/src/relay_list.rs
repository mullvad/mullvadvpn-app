use location::{CityCode, CountryCode, Location};

use std::net::Ipv4Addr;

use talpid_types::net::{OpenVpnParameters, WireguardParameters};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayList {
    pub countries: Vec<RelayListCountry>,
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
    pub position: [f64; 2],
    #[serde(skip_serializing_if = "Vec::is_empty", default)] pub relays: Vec<Relay>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relay {
    pub hostname: String,
    pub ipv4_addr_in: Ipv4Addr,
    pub ipv4_addr_exit: Ipv4Addr,
    pub include_in_country: bool,
    pub weight: u64,
    pub tunnels: RelayTunnels,
    #[serde(skip)] pub location: Option<Location>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RelayTunnels {
    pub openvpn: Vec<OpenVpnParameters>,
    pub wireguard: Vec<WireguardParameters>,
}
