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

mod proxy {
    use std::net::Ipv4Addr;

    use crate::types::{proto, FromProtobufTypeError};
    use talpid_types::net::proxy::{CustomProxy, Socks5Local, Socks5Remote, Shadowsocks, SocksAuth, Socks5};

    impl TryFrom<proto::CustomProxy> for CustomProxy {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::CustomProxy) -> Result<Self, Self::Error> {
            Ok(match value.proxy_method {
                Some(proto::custom_proxy::ProxyMethod::Socks5local(local)) => {
                    CustomProxy::Socks5(Socks5::Local(Socks5Local::try_from(local)?))
                },
                Some(proto::custom_proxy::ProxyMethod::Socks5remote(remote)) => {
                    CustomProxy::Socks5(Socks5::Remote(Socks5Remote::try_from(remote)?))
                },
                Some(proto::custom_proxy::ProxyMethod::Shadowsocks(shadowsocks)) => {
                    CustomProxy::Shadowsocks(Shadowsocks::try_from(shadowsocks)?)
                },
                None => {
                    return Err(FromProtobufTypeError::InvalidArgument("CustomProxy missing proxy_method field"));
                }
            })
        }
    }

    impl TryFrom<proto::Socks5Local> for Socks5Local {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Socks5Local) -> Result<Self, Self::Error> {
            use crate::types::conversions::net::try_transport_protocol_from_i32;
            let remote_ip = value.remote_ip.parse::<Ipv4Addr>().map_err(|_| {
                FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (local) message from protobuf",
                )
            })?;
            Ok(Socks5Local::new_with_transport_protocol(
                    (remote_ip, value.remote_port as u16),
                    value.local_port as u16,
                    try_transport_protocol_from_i32(value.remote_transport_protocol)?,
            ))
        }
    }

    impl TryFrom<proto::Socks5Remote> for Socks5Remote {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Socks5Remote) -> Result<Self, Self::Error> {
            let proto::Socks5Remote {
                ip,
                port,
                authentication,
            } = value;
            let ip = ip.parse::<Ipv4Addr>().map_err(|_| {
                FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (remote) message from protobuf",
                )
            })?;
            let port = port as u16;

            Ok(Socks5Remote::from(
                match authentication.map(SocksAuth::from) {
                    Some(auth) => Socks5Remote::new_with_authentication((ip, port), auth),
                    None => Socks5Remote::new((ip, port)),
                },
            ))
        }
    }

    impl TryFrom<proto::Shadowsocks> for Shadowsocks {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Shadowsocks) -> Result<Self, Self::Error> {
            let ip = value.ip.parse::<Ipv4Addr>().map_err(|_| {
                FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (remote) message from protobuf",
                )
            })?;

            Ok(Shadowsocks::new(
                (ip, value.port as u16),
                value.cipher,
                value.password,
            ))
        }
    }

    impl From<CustomProxy> for proto::CustomProxy {
        fn from(value: CustomProxy) -> Self {
            let proxy_method = match value {
                CustomProxy::Shadowsocks(ss) => {
                    proto::custom_proxy::ProxyMethod::Shadowsocks(
                        proto::Shadowsocks {
                            ip: ss.peer.ip().to_string(),
                            port: ss.peer.port() as u32,
                            password: ss.password,
                            cipher: ss.cipher,
                        },
                    )
                }
                CustomProxy::Socks5(Socks5::Local(Socks5Local {
                    remote_endpoint,
                    local_port,
                })) => proto::custom_proxy::ProxyMethod::Socks5local(
                    proto::Socks5Local {
                        remote_ip: remote_endpoint.address.ip().to_string(),
                        remote_port: remote_endpoint.address.port() as u32,
                        remote_transport_protocol: i32::from(proto::TransportProtocol::from(
                            remote_endpoint.protocol,
                        )),
                        local_port: local_port as u32,
                    },
                ),
                CustomProxy::Socks5(Socks5::Remote(Socks5Remote {
                    peer,
                    authentication,
                })) => proto::custom_proxy::ProxyMethod::Socks5remote(
                    proto::Socks5Remote {
                        ip: peer.ip().to_string(),
                        port: peer.port() as u32,
                        authentication: authentication.map(proto::SocksAuth::from),
                    },
                ),
            };

            proto::CustomProxy {
                proxy_method: Some(proxy_method),
            }
        }
    }

    impl From<SocksAuth> for proto::SocksAuth {
        fn from(value: SocksAuth) -> Self {
            proto::SocksAuth {
                username: value.username,
                password: value.password,
            }
        }
    }

    impl From<proto::SocksAuth> for SocksAuth {
        fn from(value: proto::SocksAuth) -> Self {
            Self {
                username: value.username,
                password: value.password,
            }
        }
    }
}
