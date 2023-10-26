use crate::types::{conversions::arg_from_str, proto, FromProtobufTypeError};
use std::net::SocketAddr;

impl From<talpid_types::net::TunnelEndpoint> for proto::TunnelEndpoint {
    fn from(endpoint: talpid_types::net::TunnelEndpoint) -> Self {
        use talpid_types::net;

        proto::TunnelEndpoint {
            address: endpoint.endpoint.address.to_string(),
            protocol: i32::from(proto::TransportProtocol::from(endpoint.endpoint.protocol)),
            tunnel_type: match endpoint.tunnel_type {
                net::TunnelType::Wireguard => i32::from(proto::TunnelType::Wireguard),
                net::TunnelType::OpenVpn => i32::from(proto::TunnelType::Openvpn),
            },
            quantum_resistant: endpoint.quantum_resistant,
            proxy: endpoint.proxy.map(|proxy_ep| proto::ProxyEndpoint {
                address: proxy_ep.endpoint.address.to_string(),
                protocol: i32::from(proto::TransportProtocol::from(proxy_ep.endpoint.protocol)),
                proxy_type: match proxy_ep.proxy_type {
                    net::proxy::ProxyType::Shadowsocks => i32::from(proto::ProxyType::Shadowsocks),
                    net::proxy::ProxyType::Custom => i32::from(proto::ProxyType::Custom),
                },
            }),
            obfuscation: endpoint.obfuscation.map(|obfuscation_endpoint| {
                proto::ObfuscationEndpoint {
                    address: obfuscation_endpoint.endpoint.address.ip().to_string(),
                    port: u32::from(obfuscation_endpoint.endpoint.address.port()),
                    protocol: i32::from(proto::TransportProtocol::from(
                        obfuscation_endpoint.endpoint.protocol,
                    )),
                    obfuscation_type: match obfuscation_endpoint.obfuscation_type {
                        net::ObfuscationType::Udp2Tcp => i32::from(proto::ObfuscationType::Udp2tcp),
                    },
                }
            }),
            entry_endpoint: endpoint.entry_endpoint.map(|entry| proto::Endpoint {
                address: entry.address.to_string(),
                protocol: i32::from(proto::TransportProtocol::from(entry.protocol)),
            }),
            tunnel_metadata: endpoint
                .tunnel_interface
                .map(|tunnel_interface| proto::TunnelMetadata { tunnel_interface }),
        }
    }
}

impl TryFrom<proto::TunnelEndpoint> for talpid_types::net::TunnelEndpoint {
    type Error = FromProtobufTypeError;

    fn try_from(endpoint: proto::TunnelEndpoint) -> Result<Self, Self::Error> {
        use talpid_types::net as talpid_net;

        Ok(talpid_net::TunnelEndpoint {
            endpoint: talpid_net::Endpoint {
                address: arg_from_str(&endpoint.address, "invalid endpoint address")?,
                protocol: try_transport_protocol_from_i32(endpoint.protocol)?,
            },
            tunnel_type: try_tunnel_type_from_i32(endpoint.tunnel_type)?,
            quantum_resistant: endpoint.quantum_resistant,
            proxy: endpoint
                .proxy
                .map(|proxy_ep| {
                    Ok(talpid_net::proxy::ProxyEndpoint {
                        endpoint: talpid_net::Endpoint {
                            address: arg_from_str(
                                &proxy_ep.address,
                                "invalid proxy endpoint address",
                            )?,
                            protocol: try_transport_protocol_from_i32(proxy_ep.protocol)?,
                        },
                        proxy_type: match proto::ProxyType::try_from(proxy_ep.proxy_type) {
                            Ok(proto::ProxyType::Shadowsocks) => {
                                talpid_net::proxy::ProxyType::Shadowsocks
                            }
                            Ok(proto::ProxyType::Custom) => talpid_net::proxy::ProxyType::Custom,
                            Err(_) => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "unknown proxy type",
                                ))
                            }
                        },
                    })
                })
                .transpose()?,
            obfuscation: endpoint
                .obfuscation
                .map(|obfs_ep| {
                    Ok(talpid_net::ObfuscationEndpoint {
                        endpoint: talpid_net::Endpoint {
                            address: SocketAddr::new(
                                arg_from_str(
                                    &obfs_ep.address,
                                    "invalid obfuscation endpoint address",
                                )?,
                                obfs_ep.port as u16,
                            ),
                            protocol: try_transport_protocol_from_i32(obfs_ep.protocol)?,
                        },
                        obfuscation_type: match proto::ObfuscationType::try_from(
                            obfs_ep.obfuscation_type,
                        ) {
                            Ok(proto::ObfuscationType::Udp2tcp) => {
                                talpid_net::ObfuscationType::Udp2Tcp
                            }
                            Err(_) => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "unknown obfuscation type",
                                ))
                            }
                        },
                    })
                })
                .transpose()?,
            entry_endpoint: endpoint
                .entry_endpoint
                .map(|entry| {
                    Ok(talpid_net::Endpoint {
                        address: arg_from_str(&entry.address, "invalid entry endpoint address")?,
                        protocol: try_transport_protocol_from_i32(entry.protocol)?,
                    })
                })
                .transpose()?,
            tunnel_interface: endpoint
                .tunnel_metadata
                .map(|tunnel_metadata| tunnel_metadata.tunnel_interface),
        })
    }
}

impl From<talpid_types::net::TransportProtocol> for proto::TransportProtocol {
    fn from(protocol: talpid_types::net::TransportProtocol) -> Self {
        match protocol {
            talpid_types::net::TransportProtocol::Udp => proto::TransportProtocol::Udp,
            talpid_types::net::TransportProtocol::Tcp => proto::TransportProtocol::Tcp,
        }
    }
}

impl From<talpid_types::net::IpVersion> for proto::IpVersion {
    fn from(version: talpid_types::net::IpVersion) -> Self {
        match version {
            talpid_types::net::IpVersion::V4 => proto::IpVersion::V4,
            talpid_types::net::IpVersion::V6 => proto::IpVersion::V6,
        }
    }
}

impl From<proto::TransportProtocol> for talpid_types::net::TransportProtocol {
    fn from(protocol: proto::TransportProtocol) -> Self {
        match protocol {
            proto::TransportProtocol::Udp => talpid_types::net::TransportProtocol::Udp,
            proto::TransportProtocol::Tcp => talpid_types::net::TransportProtocol::Tcp,
        }
    }
}

impl From<proto::IpVersion> for talpid_types::net::IpVersion {
    fn from(version: proto::IpVersion) -> Self {
        match version {
            proto::IpVersion::V4 => talpid_types::net::IpVersion::V4,
            proto::IpVersion::V6 => talpid_types::net::IpVersion::V6,
        }
    }
}

pub fn try_tunnel_type_from_i32(
    tunnel_type: i32,
) -> Result<talpid_types::net::TunnelType, FromProtobufTypeError> {
    match proto::TunnelType::try_from(tunnel_type) {
        Ok(proto::TunnelType::Openvpn) => Ok(talpid_types::net::TunnelType::OpenVpn),
        Ok(proto::TunnelType::Wireguard) => Ok(talpid_types::net::TunnelType::Wireguard),
        Err(_) => Err(FromProtobufTypeError::InvalidArgument(
            "invalid tunnel protocol",
        )),
    }
}

pub fn try_transport_protocol_from_i32(
    protocol: i32,
) -> Result<talpid_types::net::TransportProtocol, FromProtobufTypeError> {
    Ok(proto::TransportProtocol::try_from(protocol)
        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid transport protocol"))?
        .into())
}
