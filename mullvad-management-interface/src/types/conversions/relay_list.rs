use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    str::FromStr,
};

use vec1::Vec1;

use crate::types::{FromProtobufTypeError, conversions::bytes_to_pubkey, proto};

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

impl From<mullvad_types::relay_list::Relay> for proto::Relay {
    fn from(relay: mullvad_types::relay_list::Relay) -> Self {
        use mullvad_types::relay_list::RelayEndpointData as MullvadEndpointData;

        Self {
            hostname: relay.hostname,
            ipv4_addr_in: relay.ipv4_addr_in.to_string(),
            ipv6_addr_in: relay.ipv6_addr_in.map(|addr| addr.to_string()),
            include_in_country: relay.include_in_country,
            active: relay.active,
            owned: relay.owned,
            provider: relay.provider,
            weight: relay.weight,
            endpoint_data: {
                use proto::relay::RelayData;
                use proto::relay::relay_data::{Bridge, Data, OpenVpn, Wireguard, wireguard};
                let data = match relay.endpoint_data {
                    MullvadEndpointData::Wireguard(data) => {
                        let shadowsocks_extra_addr_in = data
                            .shadowsocks_extra_in_addrs()
                            .map(|addr| addr.to_string())
                            .collect();
                        let public_key = data.public_key.as_bytes().to_vec();
                        let daita = data.daita;
                        let quic = data.quic.map(wireguard::Quic::from);
                        Data::Wireguard(Wireguard {
                            public_key,
                            daita,
                            shadowsocks_extra_addr_in,
                            quic,
                            lwo: data.lwo,
                        })
                    }
                    MullvadEndpointData::Bridge => Data::Bridge(Bridge {}),
                    MullvadEndpointData::Openvpn => Data::Openvpn(OpenVpn {}),
                };
                Some(RelayData { data: Some(data) })
            },
            location: Some(proto::Location {
                country: relay.location.country,
                country_code: relay.location.country_code,
                city: relay.location.city,
                city_code: relay.location.city_code,
                latitude: relay.location.latitude,
                longitude: relay.location.longitude,
            }),
        }
    }
}

impl From<mullvad_types::relay_list::Quic> for proto::relay::relay_data::wireguard::Quic {
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

impl TryFrom<proto::relay::relay_data::wireguard::Quic> for mullvad_types::relay_list::Quic {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::relay::relay_data::wireguard::Quic) -> Result<Self, Self::Error> {
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

        let endpoint_data = {
            let data = relay
                .endpoint_data
                .and_then(|endpoint| endpoint.data)
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "invalid relay endpoint type",
                ))?;
            match data {
                proto::relay::relay_data::Data::Openvpn(_openvpn) => MullvadEndpointData::Openvpn,
                proto::relay::relay_data::Data::Bridge(_bridge) => MullvadEndpointData::Bridge,
                proto::relay::relay_data::Data::Wireguard(wireguard) => {
                    fn parse_addr(addr: &str) -> Result<IpAddr, FromProtobufTypeError> {
                        addr.parse().map_err(|_err| {
                            FromProtobufTypeError::InvalidArgument("Invalid IP address")
                        })
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
                    let data = mullvad_types::relay_list::WireguardRelayEndpointData {
                        public_key,
                        daita,
                        quic,
                        lwo: wireguard.lwo,
                        shadowsocks_extra_addr_in,
                    };
                    MullvadEndpointData::Wireguard(data)
                }
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

        let relay = MullvadRelay {
            hostname: relay.hostname,
            ipv4_addr_in: relay.ipv4_addr_in.parse().map_err(|_err| {
                FromProtobufTypeError::InvalidArgument("invalid relay IPv4 address")
            })?,
            ipv6_addr_in,
            overridden_ipv4: false,
            overridden_ipv6: false,
            include_in_country: relay.include_in_country,
            active: relay.active,
            owned: relay.owned,
            provider: relay.provider,
            weight: relay.weight,
            endpoint_data,
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
        };

        Ok(relay)
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

        Ok(mullvad_types::relay_list::WireguardEndpointData {
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
