use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    str::FromStr,
};

use mullvad_types::relay_list::{Relay, WireguardRelay};
use vec1::Vec1;

use crate::types::{FromProtobufTypeError, conversions::bytes_to_pubkey, proto};

use super::net::try_transport_protocol_from_i32;

impl From<mullvad_types::relay_list::RelayList> for proto::RelayList {
    fn from(relay_list: mullvad_types::relay_list::RelayList) -> Self {
        let mut proto_list = proto::RelayList {
            countries: vec![],
            bridge: vec![],
            bridge_endpoint: Some(proto::BridgeEndpointData::from(relay_list.bridge_endpoint)),
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

        let countries = value
            .countries
            .into_iter()
            .map(mullvad_types::relay_list::RelayListCountry::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(mullvad_types::relay_list::RelayList {
            etag: None,
            countries,
            bridge: todo!(),
            bridge_endpoint: mullvad_types::relay_list::BridgeEndpointData::try_from(bridge)?,
            wireguard: mullvad_types::relay_list::EndpointData::try_from(wireguard)?,
        })
    }
}

impl From<mullvad_types::relay_list::EndpointData> for proto::WireguardEndpointData {
    fn from(wireguard: mullvad_types::relay_list::EndpointData) -> Self {
        proto::WireguardEndpointData {
            port_ranges: wireguard
                .port_ranges
                .into_iter()
                .map(proto::PortRange::from)
                .collect(),
            ipv4_gateway: wireguard.ipv4_gateway.to_string(),
            ipv6_gateway: wireguard.ipv6_gateway.to_string(),
            shadowsocks_port_ranges: wireguard
                .shadowsocks_port_ranges
                .into_iter()
                .map(proto::PortRange::from)
                .collect(),
            udp2tcp_ports: wireguard.udp2tcp_ports.into_iter().map(u32::from).collect(),
        }
    }
}

impl From<RangeInclusive<u16>> for proto::PortRange {
    fn from(range: RangeInclusive<u16>) -> Self {
        proto::PortRange {
            first: u32::from(*range.start()),
            last: u32::from(*range.end()),
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

impl From<mullvad_types::relay_list::WireguardRelay> for proto::Relay {
    fn from(relay: mullvad_types::relay_list::WireguardRelay) -> Self {
        Self {
            hostname: relay.hostname.clone(),
            ipv4_addr_in: relay.ipv4_addr_in.to_string(),
            ipv6_addr_in: relay.ipv6_addr_in.map(|addr| addr.to_string()),
            include_in_country: relay.include_in_country,
            active: relay.active,
            owned: relay.owned,
            provider: relay.provider.clone(),
            weight: relay.weight,
            endpoint_data: {
                // TODO: Rename this type
                let mullvad_types::relay_list::WireguardRelayEndpointData {
                    public_key,
                    daita,
                    quic,
                    lwo,
                    shadowsocks_extra_addr_in,
                } = relay.endpoint_data.clone();
                let data = proto::relay::WireguardEndpoint {
                    public_key: public_key.as_bytes().to_vec(),
                    daita,
                    shadowsocks_extra_addr_in: shadowsocks_extra_addr_in
                        .iter()
                        .map(|addr| addr.to_string())
                        .collect(),
                    quic: quic.map(proto::relay::wireguard_endpoint::Quic::from),
                    lwo,
                };

                Some(data)
            },
            location: Some(proto::Location {
                country: relay.location.country.clone(),
                country_code: relay.location.country_code.clone(),
                city: relay.location.city.clone(),
                city_code: relay.location.city_code.clone(),
                latitude: relay.location.latitude,
                longitude: relay.location.longitude,
            }),
        }
    }
}

impl From<mullvad_types::relay_list::Quic> for proto::relay::wireguard_endpoint::Quic {
    fn from(quic: mullvad_types::relay_list::Quic) -> Self {
        let domain = quic.hostname().to_owned();
        let token = quic.auth_token().to_owned();
        let addr_in = quic.in_addr().map(|ip| ip.to_string()).collect();
        Self {
            domain,
            token,
            addr_in,
        }
    }
}

impl TryFrom<proto::relay::wireguard_endpoint::Quic> for mullvad_types::relay_list::Quic {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::relay::wireguard_endpoint::Quic) -> Result<Self, Self::Error> {
        let domain = value.domain;
        let token = value.token;
        fn parse_addr(addr: String) -> Result<IpAddr, FromProtobufTypeError> {
            addr.parse()
                .map_err(|_err| FromProtobufTypeError::InvalidArgument("Invalid IP address"))
        }
        let addr_in = value
            .addr_in
            .into_iter()
            .map(parse_addr)
            .collect::<Result<Vec<IpAddr>, FromProtobufTypeError>>()?;
        let addr_in = Vec1::try_from_vec(addr_in)
            .map_err(|_err| FromProtobufTypeError::InvalidArgument("Invalid QUIC object"))?;
        Ok(Self::new(addr_in, token, domain))
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
            .map(mullvad_types::relay_list::WireguardRelay::try_from)
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

impl TryFrom<proto::Relay> for mullvad_types::relay_list::WireguardRelay {
    type Error = FromProtobufTypeError;

    fn try_from(relay: proto::Relay) -> Result<Self, Self::Error> {
        use mullvad_types::{
            location::Location as MullvadLocation, relay_list::WireguardRelay as MullvadRelay,
        };

        let endpoint_data = {
            let wireguard = relay
                .endpoint_data
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "invalid relay endpoint type",
                ))?;
            fn parse_addr(addr: &str) -> Result<IpAddr, FromProtobufTypeError> {
                addr.parse()
                    .map_err(|_err| FromProtobufTypeError::InvalidArgument("Invalid IP address"))
            }

            let public_key = bytes_to_pubkey(&wireguard.public_key)?;
            let daita = wireguard.daita;
            let quic = wireguard
                .quic
                .map(mullvad_types::relay_list::Quic::try_from)
                .transpose()?;
            let shadowsocks_extra_addr_in = wireguard
                .shadowsocks_extra_addr_in
                .iter()
                .map(String::as_ref)
                .map(parse_addr)
                .collect::<Result<HashSet<IpAddr>, FromProtobufTypeError>>()?;
            mullvad_types::relay_list::WireguardRelayEndpointData {
                public_key,
                daita,
                quic,
                lwo: wireguard.lwo,
                shadowsocks_extra_addr_in,
            }
        };

        let ipv6_addr_in = relay
            .ipv6_addr_in
            .map(|addr| {
                addr.parse().map_err(|_err| {
                    FromProtobufTypeError::InvalidArgument("invalid relay IPv6 address")
                })
            })
            .transpose()?;

        let relay = WireguardRelay::new(
            false,
            false,
            relay.include_in_country,
            relay.owned,
            relay.provider,
            endpoint_data,
            Relay {
                hostname: relay.hostname.clone(),
                ipv4_addr_in: relay.ipv4_addr_in.parse().map_err(|_err| {
                    FromProtobufTypeError::InvalidArgument("invalid relay IPv4 address")
                })?,
                ipv6_addr_in,
                active: relay.active,
                weight: relay.weight,
                location: relay
                    .location
                    .map(|location| MullvadLocation {
                        country: location.country,
                        country_code: location.country_code,
                        city: location.city,
                        city_code: location.city_code,
                        latitude: location.latitude,
                        longitude: location.longitude,
                    })
                    .ok_or("missing relay location")
                    .map_err(FromProtobufTypeError::InvalidArgument)?,
            },
        );
        Ok(relay)
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

impl TryFrom<proto::WireguardEndpointData> for mullvad_types::relay_list::EndpointData {
    type Error = FromProtobufTypeError;

    fn try_from(wireguard: proto::WireguardEndpointData) -> Result<Self, FromProtobufTypeError> {
        let port_ranges = wireguard
            .port_ranges
            .into_iter()
            .map(RangeInclusive::try_from)
            .collect::<Result<Vec<_>, FromProtobufTypeError>>()?;

        let ipv4_gateway = Ipv4Addr::from_str(&wireguard.ipv4_gateway)
            .map_err(|_| FromProtobufTypeError::InvalidArgument("Invalid IPv4 gateway"))?;
        let ipv6_gateway = Ipv6Addr::from_str(&wireguard.ipv6_gateway)
            .map_err(|_| FromProtobufTypeError::InvalidArgument("Invalid IPv6 gateway"))?;

        let shadowsocks_port_ranges = wireguard
            .shadowsocks_port_ranges
            .into_iter()
            .map(RangeInclusive::try_from)
            .collect::<Result<Vec<_>, FromProtobufTypeError>>()?;

        let udp2tcp_ports = wireguard
            .udp2tcp_ports
            .into_iter()
            .map(|port| {
                u16::try_from(port)
                    .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid udp2tcp port"))
            })
            .collect::<Result<Vec<u16>, FromProtobufTypeError>>()?;

        Ok(mullvad_types::relay_list::EndpointData {
            port_ranges,
            ipv4_gateway,
            ipv6_gateway,
            shadowsocks_port_ranges,
            udp2tcp_ports,
        })
    }
}

impl TryFrom<proto::PortRange> for RangeInclusive<u16> {
    type Error = FromProtobufTypeError;

    fn try_from(range: proto::PortRange) -> Result<Self, Self::Error> {
        let first = u16::try_from(range.first)
            .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid port"))?;
        let last = u16::try_from(range.last)
            .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid port"))?;
        Ok(first..=last)
    }
}
