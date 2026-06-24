use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub type CountryCode = String;
pub type CityCode = String;
pub type Hostname = String;

/// Describes the physical location of a [`crate::relay_list::Relay`] as returned by the API.
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
    pub fn distance_from(&self, other: impl Into<Coordinates>) -> f64 {
        let other: Coordinates = other.into();
        haversine_dist_deg(
            self.latitude,
            self.longitude,
            other.latitude,
            other.longitude,
        )
    }

    pub fn has_same_city(&self, other: &Self) -> bool {
        self.country_code == other.country_code && self.city_code == other.city_code
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl From<&Location> for Coordinates {
    fn from(location: &Location) -> Self {
        Self {
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<Location> for Coordinates {
    fn from(location: Location) -> Self {
        Coordinates::from(&location)
    }
}

impl From<&GeoIpLocation> for Coordinates {
    fn from(location: &GeoIpLocation) -> Self {
        Self {
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<GeoIpLocation> for Coordinates {
    fn from(location: GeoIpLocation) -> Self {
        Coordinates::from(&location)
    }
}

impl Coordinates {
    /// Computes the approximate midpoint of a set of locations.
    ///
    /// This works by calculating the mean Cartesian coordinates, and converting them
    /// back to spherical coordinates. This is approximate, because the semi-minor (polar)
    /// axis is assumed to equal the semi-major (equatorial) axis.
    ///
    /// <https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates>
    pub fn midpoint(locations: &[Location]) -> Self {
        Self::midpoint_inner(locations.iter().map(Coordinates::from))
    }

    fn midpoint_inner(locations: impl std::iter::Iterator<Item = Coordinates>) -> Self {
        let mut x = 0f64;
        let mut y = 0f64;
        let mut z = 0f64;

        let mut count = 0;

        for location in locations {
            let cos_lat = location.latitude.to_radians().cos();
            let sin_lat = location.latitude.to_radians().sin();
            let cos_lon = location.longitude.to_radians().cos();
            let sin_lon = location.longitude.to_radians().sin();
            x += cos_lat * cos_lon;
            y += cos_lat * sin_lon;
            z += sin_lat;
            count += 1;
        }
        let inv_total_weight = 1f64 / (count as f64);
        x *= inv_total_weight;
        y *= inv_total_weight;
        z *= inv_total_weight;

        let longitude = y.atan2(x);
        let hypotenuse = (x * x + y * y).sqrt();
        let latitude = z.atan2(hypotenuse);

        Coordinates {
            latitude: latitude.to_degrees(),
            longitude: longitude.to_degrees(),
        }
    }
}

/// Takes input as latitude and longitude degrees.
fn haversine_dist_deg(lat: f64, lon: f64, other_lat: f64, other_lon: f64) -> f64 {
    haversine_dist_rad(
        lat.to_radians(),
        lon.to_radians(),
        other_lat.to_radians(),
        other_lon.to_radians(),
    )
}
/// Implemented as per <https://en.wikipedia.org/wiki/Haversine_formula> and <https://rosettacode.org/wiki/Haversine_formula#Rust>
/// Takes input as radians, outputs kilometers.
fn haversine_dist_rad(lat: f64, lon: f64, other_lat: f64, other_lon: f64) -> f64 {
    let d_lat = lat - other_lat;
    let d_lon = lon - other_lon;
    // Computing the haversine between two points
    let haversine =
        (d_lat / 2.0).sin().powi(2) + (d_lon / 2.0).sin().powi(2) * lat.cos() * other_lat.cos();

    // using the haversine to compute the distance between two points
    haversine.sqrt().asin() * 2.0 * RAIDUS_OF_EARTH
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoIpLocation {
    /// A geographic coordinate that specifies the north-south position of a point on the surface of the Earth as an angle ranging from `-90°` (south pole) to `+90°` (north pole) with `0°` at the equator.
    pub latitude: f64,
    /// A geographic coordinate that specifies the east-west position of a point on the surface of the Earth.
    /// Longitude is given as an angular measurement with `0°` at the Prime Meridian, ranging from `−180°` westward to `+180°` eastward.
    pub longitude: f64,
    /// Public IPv4 address ("Out-IP").
    pub ipv4: Option<Ipv4Addr>,
    /// Public IPv6 address ("Out-IP").
    pub ipv6: Option<Ipv6Addr>,
    /// If public IP address(es) are Mullvad IPs.
    pub mullvad_exit_ip: bool,
    /// Hostname of the exit relay. e.g. `se-got-wg-101`.
    pub hostname: Option<String>,
    /// City code of the exit relay. e.g. `got` for "Gothenburg".
    pub city: Option<String>,
    /// Country code of the exit relay. e.g. `se` for "Sweden".
    pub country: String,
    /// Hostname of the entry relay. e.g. `se-got-wg-101`.
    ///
    /// This field is `Some` when traffic is routed through an entry relay.
    pub entry_hostname: Option<String>,
    /// City code of the entry relay. e.g. `got` for "Gothenburg".
    ///
    /// This field is `Some` when traffic is routed through an entry relay.
    pub entry_city: Option<String>,
    ///  Country code of the entry relay. e.g. `se` for "Sweden".
    ///
    /// This field is `Some` when traffic is routed through an entry relay.
    pub entry_country: Option<String>,
}

impl From<AmIMullvad> for GeoIpLocation {
    fn from(location: AmIMullvad) -> GeoIpLocation {
        let (ipv4, ipv6) = match location.ip {
            IpAddr::V4(v4) => (Some(v4), None),
            IpAddr::V6(v6) => (None, Some(v6)),
        };

        GeoIpLocation {
            latitude: location.latitude,
            longitude: location.longitude,
            ipv4,
            ipv6,
            mullvad_exit_ip: location.mullvad_exit_ip,
            hostname: None,
            city: location.city,
            country: location.country,
            entry_hostname: None,
            entry_city: None,
            entry_country: None,
        }
    }
}

pub struct LocationEventData {
    /// Keep track of which request led to this event being triggered
    pub request_id: usize,
    /// New location information
    pub location: GeoIpLocation,
}

#[cfg(test)]
mod tests {
    use super::Coordinates;

    impl Coordinates {
        fn equal(&self, other: Coordinates) -> bool {
            const EPS: f64 = 0.1;
            (self.latitude - other.latitude).abs() < EPS
                && (self.longitude - other.longitude).abs() < EPS
        }
    }

    #[test]
    fn test_haversine_dist_deg() {
        use super::haversine_dist_deg;
        assert_eq!(
            haversine_dist_deg(36.12, -86.67, 33.94, -118.4),
            2_887.259_950_607_111
        );
        assert_eq!(
            haversine_dist_deg(90.0, 5.0, 90.0, 79.0),
            0.0000000000004696822692507987
        );
        assert_eq!(haversine_dist_deg(0.0, 0.0, 0.0, 0.0), 0.0);
        assert_eq!(haversine_dist_deg(49.0, 12.0, 49.0, 12.0), 0.0);
        assert_eq!(haversine_dist_deg(6.0, 27.0, 7.0, 27.0), 111.22634257109462);
        assert_eq!(
            haversine_dist_deg(0.0, 179.5, 0.0, -179.5),
            111.22634257109495
        );
    }

    #[test]
    fn test_midpoint() {
        assert!(
            Coordinates::midpoint_inner(
                [
                    Coordinates {
                        latitude: 0.0,
                        longitude: 90.0,
                    },
                    Coordinates {
                        latitude: 90.0,
                        longitude: 0.0,
                    },
                ]
                .into_iter()
            )
            .equal(Coordinates {
                latitude: 45.0,
                longitude: 90.0,
            })
        );

        assert!(
            Coordinates::midpoint_inner(
                [
                    Coordinates {
                        latitude: -20.0,
                        longitude: 90.0,
                    },
                    Coordinates {
                        latitude: -20.0,
                        longitude: -90.0,
                    },
                ]
                .into_iter()
            )
            .equal(Coordinates {
                latitude: -90.0,
                longitude: 0.0,
            })
        );
    }
}
