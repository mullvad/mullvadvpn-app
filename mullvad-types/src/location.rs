use std::net::IpAddr;

pub type CountryCode = String;
pub type CityCode = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub country: String,
    pub country_code: CountryCode,
    pub city: String,
    pub city_code: CityCode,
    pub position: [f64; 2],
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

impl GeoIpLocation {
    // pub fn ip(&self) -> Option<IpAddr> {
    //     IpAddr::from_str(&self.ip).ok()
    // }

    pub fn position(&self) -> [f64; 2] {
        [self.latitude, self.longitude]
    }
}
