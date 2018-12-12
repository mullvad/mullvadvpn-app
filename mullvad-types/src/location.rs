use serde::{Deserialize, Serialize};
use std::net::IpAddr;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoIpLocation {
    pub ip: Option<IpAddr>,
    pub country: String,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub mullvad_exit_ip: bool,
    #[serde(default)]
    pub hostname: Option<String>,
}
