use crate::types::{FromProtobufTypeError, conversions::arg_from_str, proto};

impl From<mullvad_types::location::GeoIpLocation> for proto::GeoIpLocation {
    fn from(geoip: mullvad_types::location::GeoIpLocation) -> proto::GeoIpLocation {
        proto::GeoIpLocation {
            ipv4: geoip.ipv4.map(|ip| ip.to_string()),
            ipv6: geoip.ipv6.map(|ip| ip.to_string()),
            country: geoip.country,
            city: geoip.city,
            latitude: geoip.latitude,
            longitude: geoip.longitude,
            mullvad_exit_ip: geoip.mullvad_exit_ip,
            hostname: geoip.hostname,
            bridge_hostname: todo!("remove"),
            entry_hostname: geoip.entry_hostname,
            obfuscator_hostname: geoip.obfuscator_hostname,
        }
    }
}

impl TryFrom<proto::GeoIpLocation> for mullvad_types::location::GeoIpLocation {
    type Error = FromProtobufTypeError;

    fn try_from(geoip: proto::GeoIpLocation) -> Result<Self, Self::Error> {
        Ok(mullvad_types::location::GeoIpLocation {
            ipv4: geoip
                .ipv4
                .map(|addr| arg_from_str(&addr, "invalid IPv4 address"))
                .transpose()?,
            ipv6: geoip
                .ipv6
                .map(|addr| arg_from_str(&addr, "invalid IPv6 address"))
                .transpose()?,
            country: geoip.country,
            city: geoip.city,
            latitude: geoip.latitude,
            longitude: geoip.longitude,
            mullvad_exit_ip: geoip.mullvad_exit_ip,
            hostname: geoip.hostname,
            entry_hostname: geoip.entry_hostname,
            obfuscator_hostname: geoip.obfuscator_hostname,
        })
    }
}
