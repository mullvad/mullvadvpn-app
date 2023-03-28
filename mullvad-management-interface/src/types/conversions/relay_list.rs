use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::types::{
    conversions::{bytes_to_pubkey, option_from_proto_string, to_proto_any, try_from_proto_any},
    proto, FromProtobufTypeError,
};

use super::net::try_transport_protocol_from_i32;

impl From<mullvad_types::relay_list::RelayList> for proto::RelayList {
    fn from(relay_list: mullvad_types::relay_list::RelayList) -> Self {
        let mut proto_list = proto::RelayList {
            countries: vec![],
            openvpn: Some(proto::OpenVpnEndpointData::from(relay_list.openvpn)),
            bridge: Some(proto::BridgeEndpointData::from(relay_list.bridge)),
            wireguard: Some(proto::WireguardEndpointData::from(relay_list.wireguard)),
        };
        proto_list.countries = relay_list
            .countries
            .into_iter()
            .map(proto::RelayListCountry::from)
            .collect();
        proto_list
    }
}

impl From<mullvad_types::relay_list::OpenVpnEndpointData> for proto::OpenVpnEndpointData {
    fn from(openvpn: mullvad_types::relay_list::OpenVpnEndpointData) -> Self {
        proto::OpenVpnEndpointData {
            endpoints: openvpn
                .ports
                .into_iter()
                .map(|endpoint| proto::OpenVpnEndpoint {
                    port: u32::from(endpoint.port),
                    protocol: proto::TransportProtocol::from(endpoint.protocol) as i32,
                })
                .collect(),
        }
    }
}

impl From<mullvad_types::relay_list::BridgeEndpointData> for proto::BridgeEndpointData {
    fn from(bridge: mullvad_types::relay_list::BridgeEndpointData) -> Self {
        proto::BridgeEndpointData {
            shadowsocks: bridge
                .shadowsocks
                .into_iter()
                .map(|endpoint| proto::ShadowsocksEndpointData {
                    port: u32::from(endpoint.port),
                    cipher: endpoint.cipher,
                    password: endpoint.password,
                    protocol: proto::TransportProtocol::from(endpoint.protocol) as i32,
                })
                .collect(),
        }
    }
}

impl From<mullvad_types::relay_list::WireguardEndpointData> for proto::WireguardEndpointData {
    fn from(wireguard: mullvad_types::relay_list::WireguardEndpointData) -> Self {
        proto::WireguardEndpointData {
            port_ranges: wireguard
                .port_ranges
                .into_iter()
                .map(|(first, last)| proto::PortRange {
                    first: u32::from(first),
                    last: u32::from(last),
                })
                .collect(),
            ipv4_gateway: wireguard.ipv4_gateway.to_string(),
            ipv6_gateway: wireguard.ipv6_gateway.to_string(),
            udp2tcp_ports: wireguard.udp2tcp_ports.into_iter().map(u32::from).collect(),
        }
    }
}

impl From<mullvad_types::relay_list::RelayListCountry> for proto::RelayListCountry {
    fn from(country: mullvad_types::relay_list::RelayListCountry) -> Self {
        let mut proto_country = proto::RelayListCountry {
            name: country.name,
            code: country.code,
            cities: Vec::with_capacity(country.cities.len()),
        };

        for city in country.cities.into_iter() {
            proto_country.cities.push(proto::RelayListCity {
                name: city.name,
                code: city.code,
                latitude: city.latitude,
                longitude: city.longitude,
                relays: city.relays.into_iter().map(proto::Relay::from).collect(),
            });
        }

        proto_country
    }
}

impl From<mullvad_types::relay_list::Relay> for proto::Relay {
    fn from(relay: mullvad_types::relay_list::Relay) -> Self {
        use mullvad_types::relay_list::RelayEndpointData as MullvadEndpointData;

        Self {
            hostname: relay.hostname,
            ipv4_addr_in: relay.ipv4_addr_in.to_string(),
            ipv6_addr_in: relay
                .ipv6_addr_in
                .map(|addr| addr.to_string())
                .unwrap_or_default(),
            include_in_country: relay.include_in_country,
            active: relay.active,
            owned: relay.owned,
            provider: relay.provider,
            weight: relay.weight,
            endpoint_type: match &relay.endpoint_data {
                MullvadEndpointData::Openvpn => proto::relay::RelayType::Openvpn as i32,
                MullvadEndpointData::Bridge => proto::relay::RelayType::Bridge as i32,
                MullvadEndpointData::Wireguard(_) => proto::relay::RelayType::Wireguard as i32,
            },
            endpoint_data: match relay.endpoint_data {
                MullvadEndpointData::Wireguard(data) => Some(to_proto_any(
                    "mullvad_daemon.management_interface/WireguardRelayEndpointData",
                    proto::WireguardRelayEndpointData {
                        public_key: data.public_key.as_bytes().to_vec(),
                    },
                )),
                _ => None,
            },
            location: relay.location.map(|location| proto::Location {
                country: location.country,
                country_code: location.country_code,
                city: location.city,
                city_code: location.city_code,
                latitude: location.latitude,
                longitude: location.longitude,
            }),
        }
    }
}

impl TryFrom<proto::RelayList> for mullvad_types::relay_list::RelayList {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::RelayList) -> Result<Self, Self::Error> {
        let wireguard = value
            .wireguard
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing wireguard data",
            ))?;
        let bridge = value.bridge.ok_or(FromProtobufTypeError::InvalidArgument(
            "missing bridge data",
        ))?;
        let openvpn = value.openvpn.ok_or(FromProtobufTypeError::InvalidArgument(
            "missing openvpn data",
        ))?;

        let countries = value
            .countries
            .into_iter()
            .map(mullvad_types::relay_list::RelayListCountry::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(mullvad_types::relay_list::RelayList {
            etag: None,
            countries,
            openvpn: mullvad_types::relay_list::OpenVpnEndpointData::try_from(openvpn)?,
            bridge: mullvad_types::relay_list::BridgeEndpointData::try_from(bridge)?,
            wireguard: mullvad_types::relay_list::WireguardEndpointData::try_from(wireguard)?,
        })
    }
}

impl TryFrom<proto::RelayListCountry> for mullvad_types::relay_list::RelayListCountry {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::RelayListCountry) -> Result<Self, Self::Error> {
        let cities = value
            .cities
            .into_iter()
            .map(mullvad_types::relay_list::RelayListCity::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(mullvad_types::relay_list::RelayListCountry {
            cities,
            code: value.code,
            name: value.name,
        })
    }
}

impl TryFrom<proto::RelayListCity> for mullvad_types::relay_list::RelayListCity {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::RelayListCity) -> Result<Self, Self::Error> {
        let relays = value
            .relays
            .into_iter()
            .map(mullvad_types::relay_list::Relay::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(mullvad_types::relay_list::RelayListCity {
            code: value.code,
            latitude: value.latitude,
            longitude: value.longitude,
            name: value.name,
            relays,
        })
    }
}

impl TryFrom<proto::Relay> for mullvad_types::relay_list::Relay {
    type Error = FromProtobufTypeError;

    fn try_from(relay: proto::Relay) -> Result<Self, Self::Error> {
        use mullvad_types::{
            location::Location as MullvadLocation,
            relay_list::{Relay as MullvadRelay, RelayEndpointData as MullvadEndpointData},
        };

        let endpoint_data = match relay.endpoint_type {
            i if i == proto::relay::RelayType::Openvpn as i32 => MullvadEndpointData::Openvpn,
            i if i == proto::relay::RelayType::Bridge as i32 => MullvadEndpointData::Bridge,
            i if i == proto::relay::RelayType::Wireguard as i32 => {
                let data = relay
                    .endpoint_data
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "missing endpoint wg data",
                    ))?;
                let data: proto::WireguardRelayEndpointData = try_from_proto_any(
                    "mullvad_daemon.management_interface/WireguardRelayEndpointData",
                    data,
                )
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "invalid endpoint wg data",
                ))?;
                MullvadEndpointData::Wireguard(
                    mullvad_types::relay_list::WireguardRelayEndpointData {
                        public_key: bytes_to_pubkey(&data.public_key)?,
                    },
                )
            }
            _ => {
                return Err(FromProtobufTypeError::InvalidArgument(
                    "invalid relay endpoint type",
                ))
            }
        };

        let ipv6_addr_in = option_from_proto_string(relay.ipv6_addr_in)
            .map(|addr| {
                addr.parse().map_err(|_err| {
                    FromProtobufTypeError::InvalidArgument("invalid relay IPv6 address")
                })
            })
            .transpose()?;

        Ok(MullvadRelay {
            hostname: relay.hostname,
            ipv4_addr_in: relay.ipv4_addr_in.parse().map_err(|_err| {
                FromProtobufTypeError::InvalidArgument("invalid relay IPv4 address")
            })?,
            ipv6_addr_in,
            include_in_country: relay.include_in_country,
            active: relay.active,
            owned: relay.owned,
            provider: relay.provider,
            weight: relay.weight,
            endpoint_data,
            location: relay.location.map(|location| MullvadLocation {
                country: location.country,
                country_code: location.country_code,
                city: location.city,
                city_code: location.city_code,
                latitude: location.latitude,
                longitude: location.longitude,
            }),
        })
    }
}

impl TryFrom<proto::OpenVpnEndpointData> for mullvad_types::relay_list::OpenVpnEndpointData {
    type Error = FromProtobufTypeError;

    fn try_from(openvpn: proto::OpenVpnEndpointData) -> Result<Self, FromProtobufTypeError> {
        let ports = openvpn
            .endpoints
            .into_iter()
            .map(mullvad_types::relay_list::OpenVpnEndpoint::try_from)
            .collect::<Result<Vec<_>, FromProtobufTypeError>>()?;

        Ok(mullvad_types::relay_list::OpenVpnEndpointData { ports })
    }
}

impl TryFrom<proto::OpenVpnEndpoint> for mullvad_types::relay_list::OpenVpnEndpoint {
    type Error = FromProtobufTypeError;

    fn try_from(openvpn: proto::OpenVpnEndpoint) -> Result<Self, FromProtobufTypeError> {
        Ok(mullvad_types::relay_list::OpenVpnEndpoint {
            port: u16::try_from(openvpn.port)
                .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid port"))?,
            protocol: try_transport_protocol_from_i32(openvpn.protocol)?,
        })
    }
}

impl TryFrom<proto::BridgeEndpointData> for mullvad_types::relay_list::BridgeEndpointData {
    type Error = FromProtobufTypeError;

    fn try_from(bridge: proto::BridgeEndpointData) -> Result<Self, FromProtobufTypeError> {
        let shadowsocks = bridge
            .shadowsocks
            .into_iter()
            .map(mullvad_types::relay_list::ShadowsocksEndpointData::try_from)
            .collect::<Result<Vec<_>, FromProtobufTypeError>>()?;

        Ok(mullvad_types::relay_list::BridgeEndpointData { shadowsocks })
    }
}

impl TryFrom<proto::ShadowsocksEndpointData>
    for mullvad_types::relay_list::ShadowsocksEndpointData
{
    type Error = FromProtobufTypeError;

    fn try_from(
        shadowsocks: proto::ShadowsocksEndpointData,
    ) -> Result<Self, FromProtobufTypeError> {
        Ok(mullvad_types::relay_list::ShadowsocksEndpointData {
            port: u16::try_from(shadowsocks.port)
                .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid port"))?,
            cipher: shadowsocks.cipher,
            password: shadowsocks.password,
            protocol: try_transport_protocol_from_i32(shadowsocks.protocol)?,
        })
    }
}

impl TryFrom<proto::WireguardEndpointData> for mullvad_types::relay_list::WireguardEndpointData {
    type Error = FromProtobufTypeError;

    fn try_from(wireguard: proto::WireguardEndpointData) -> Result<Self, FromProtobufTypeError> {
        let port_ranges = wireguard
            .port_ranges
            .into_iter()
            .map(|range| {
                let first = u16::try_from(range.first)
                    .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid wg port"))?;
                let last = u16::try_from(range.last)
                    .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid wg port"))?;
                Ok((first, last))
            })
            .collect::<Result<Vec<(u16, u16)>, FromProtobufTypeError>>()?;

        let ipv4_gateway = Ipv4Addr::from_str(&wireguard.ipv4_gateway)
            .map_err(|_| FromProtobufTypeError::InvalidArgument("Invalid IPv4 gateway"))?;
        let ipv6_gateway = Ipv6Addr::from_str(&wireguard.ipv6_gateway)
            .map_err(|_| FromProtobufTypeError::InvalidArgument("Invalid IPv6 gateway"))?;

        let udp2tcp_ports = wireguard
            .udp2tcp_ports
            .into_iter()
            .map(|port| {
                u16::try_from(port)
                    .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid udp2tcp port"))
            })
            .collect::<Result<Vec<u16>, FromProtobufTypeError>>()?;

        Ok(mullvad_types::relay_list::WireguardEndpointData {
            port_ranges,
            ipv4_gateway,
            ipv6_gateway,
            udp2tcp_ports,
        })
    }
}
