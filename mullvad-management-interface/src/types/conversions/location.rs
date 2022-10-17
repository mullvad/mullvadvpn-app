use crate::types::{
    conversions::{arg_from_str, option_from_proto_string},
    proto, FromProtobufTypeError,
};

impl From<mullvad_types::location::GeoIpLocation> for proto::GeoIpLocation {
    fn from(geoip: mullvad_types::location::GeoIpLocation) -> proto::GeoIpLocation {
        proto::GeoIpLocation {
            ipv4: geoip.ipv4.map(|ip| ip.to_string()).unwrap_or_default(),
            ipv6: geoip.ipv6.map(|ip| ip.to_string()).unwrap_or_default(),
            country: geoip.country,
            city: geoip.city.unwrap_or_default(),
            latitude: geoip.latitude,
            longitude: geoip.longitude,
            mullvad_exit_ip: geoip.mullvad_exit_ip,
            hostname: geoip.hostname.unwrap_or_default(),
            bridge_hostname: geoip.bridge_hostname.unwrap_or_default(),
            entry_hostname: geoip.entry_hostname.unwrap_or_default(),
            obfuscator_hostname: geoip.obfuscator_hostname.unwrap_or_default(),
        }
    }
}

impl TryFrom<proto::GeoIpLocation> for mullvad_types::location::GeoIpLocation {
    type Error = FromProtobufTypeError;

    fn try_from(geoip: proto::GeoIpLocation) -> Result<Self, Self::Error> {
        Ok(mullvad_types::location::GeoIpLocation {
            ipv4: option_from_proto_string(geoip.ipv4)
                .map(|addr| arg_from_str(&addr, "invalid IPv4 address"))
                .transpose()?,
            ipv6: option_from_proto_string(geoip.ipv6)
                .map(|addr| arg_from_str(&addr, "invalid IPv6 address"))
                .transpose()?,
            country: geoip.country,
            city: option_from_proto_string(geoip.city),
            latitude: geoip.latitude,
            longitude: geoip.longitude,
            mullvad_exit_ip: geoip.mullvad_exit_ip,
            hostname: option_from_proto_string(geoip.hostname),
            bridge_hostname: option_from_proto_string(geoip.bridge_hostname),
            entry_hostname: option_from_proto_string(geoip.entry_hostname),
            obfuscator_hostname: option_from_proto_string(geoip.obfuscator_hostname),
        })
    }
}
