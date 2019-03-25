use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub type CountryCode = String;
pub type CityCode = String;
pub type Hostname = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub country: String,
    pub country_code: CountryCode,
    pub city: String,
    pub city_code: CityCode,
    pub latitude: f64,
    pub longitude: f64,
}

/// The response from the am.i.mullvad.net location service.
#[derive(Debug, Deserialize)]
pub struct AmIMullvad {
    pub ip: IpAddr,
    pub country: String,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub mullvad_exit_ip: bool,
}

/// GeoIP information exposed from the daemon to frontends.
#[derive(Debug, Serialize, Deserialize)]
pub struct GeoIpLocation {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub country: String,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub mullvad_exit_ip: bool,
    pub hostname: Option<String>,
}

impl From<AmIMullvad> for GeoIpLocation {
    fn from(location: AmIMullvad) -> GeoIpLocation {
        let (ipv4, ipv6) = match location.ip {
            IpAddr::V4(v4) => (Some(v4), None),
            IpAddr::V6(v6) => (None, Some(v6)),
        };
        GeoIpLocation {
            ipv4,
            ipv6,
            country: location.country,
            city: location.city,
            latitude: location.latitude,
            longitude: location.longitude,
            mullvad_exit_ip: location.mullvad_exit_ip,
            hostname: None,
        }
    }
}
