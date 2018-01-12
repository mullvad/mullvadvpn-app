use std::net::IpAddr;

pub type CountryCode = String;
pub type CityCode = String;

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
    pub ip: IpAddr,
    pub country: String,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub mullvad_exit_ip: bool,
}
