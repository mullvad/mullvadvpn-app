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

const RAIDUS_OF_EARTH: f64 = 6372.8;

impl Location {
    pub fn distance_from(&self, other: &Location) -> f64 {
        Self::haversine_distance(
            self.latitude,
            self.longitude,
            other.latitude,
            other.longitude,
        )
    }

    /// Implemented as per https://en.wikipedia.org/wiki/Haversine_formula and https://rosettacode.org/wiki/Haversine_formula#Rust
    fn haversine_distance(lat: f64, lon: f64, other_lat: f64, other_lon: f64) -> f64 {
        let d_lat = (lat - other_lat).to_radians();
        let d_lon = (lon - other_lon).to_radians();
        // Computing the haversine between two points
        ((d_lat/2.0).sin().powi(2) + (d_lon/2.0).sin().powi(2) * lat.to_radians().cos() * other_lat.to_radians().cos())
            // using the haversine to compute the distance between two points
            .sqrt().asin()
            * 2.0
            * RAIDUS_OF_EARTH
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_haversine_dist() {
        assert_eq!(
            super::Location::haversine_distance(36.12, -86.67, 33.94, -118.40),
            2887.2599506071106
        );
    }
}
