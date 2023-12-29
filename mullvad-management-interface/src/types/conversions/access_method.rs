/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::access_method;

    impl From<&access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: &access_method::Settings) -> Self {
            Self {
                access_method_settings: settings
                    .access_method_settings
                    .iter()
                    .map(|method| method.clone().into())
                    .collect(),
            }
        }
    }

    impl From<access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: access_method::Settings) -> Self {
            proto::ApiAccessMethodSettings::from(&settings)
        }
    }

    impl TryFrom<proto::ApiAccessMethodSettings> for access_method::Settings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::ApiAccessMethodSettings) -> Result<Self, Self::Error> {
            Ok(Self {
                access_method_settings: settings
                    .access_method_settings
                    .iter()
                    .map(access_method::AccessMethodSetting::try_from)
                    .collect::<Result<Vec<access_method::AccessMethodSetting>, _>>()?,
            })
        }
    }
}

/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethod`] type to the internal
/// [`mullvad_types::access_method::AccessMethodSetting`] data type.
mod data {
    use std::net::Ipv4Addr;

    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::access_method::{
        AccessMethod, AccessMethodSetting, BuiltInAccessMethod, Id,
    };
    use talpid_types::net::proxy::{
        CustomProxy, Shadowsocks, Socks5Local, Socks5Remote, SocksAuth,
    };

    impl TryFrom<proto::AccessMethodSetting> for AccessMethodSetting {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::AccessMethodSetting) -> Result<Self, Self::Error> {
            let id = value
                .id
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))
                .and_then(Id::try_from)?;
            let name = value.name;
            let enabled = value.enabled;
            let access_method = value
                .access_method
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))
                .and_then(AccessMethod::try_from)?;

            Ok(AccessMethodSetting::with_id(
                id,
                name,
                enabled,
                access_method,
            ))
        }
    }

    impl From<AccessMethodSetting> for proto::AccessMethodSetting {
        fn from(value: AccessMethodSetting) -> Self {
            let id = proto::Uuid::from(value.get_id());
            let name = value.get_name();
            let enabled = value.enabled();
            proto::AccessMethodSetting {
                id: Some(id),
                name,
                enabled,
                access_method: Some(proto::AccessMethod::from(value.access_method)),
            }
        }
    }

    impl TryFrom<proto::AccessMethod> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::AccessMethod) -> Result<Self, Self::Error> {
            let access_method =
                value
                    .access_method
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Could not deserialize Access Method from protobuf",
                    ))?;

            Ok(match access_method {
                proto::access_method::AccessMethod::Direct(direct) => AccessMethod::from(direct),
                proto::access_method::AccessMethod::Bridges(bridge) => AccessMethod::from(bridge),
                proto::access_method::AccessMethod::Socks5local(sockslocal) => {
                    AccessMethod::try_from(sockslocal)?
                }
                proto::access_method::AccessMethod::Socks5remote(socksremote) => {
                    AccessMethod::try_from(socksremote)?
                }
                proto::access_method::AccessMethod::Shadowsocks(shadowsocks) => {
                    AccessMethod::try_from(shadowsocks)?
                }
            })
        }
    }

    impl From<AccessMethod> for proto::AccessMethod {
        fn from(value: AccessMethod) -> Self {
            match value {
                AccessMethod::Custom(value) => proto::AccessMethod::from(value),
                AccessMethod::BuiltIn(value) => proto::AccessMethod::from(value),
            }
        }
    }

    impl From<proto::access_method::Direct> for AccessMethod {
        fn from(_value: proto::access_method::Direct) -> Self {
            AccessMethod::from(BuiltInAccessMethod::Direct)
        }
    }

    impl From<proto::access_method::Bridges> for AccessMethod {
        fn from(_value: proto::access_method::Bridges) -> Self {
            AccessMethod::from(BuiltInAccessMethod::Bridge)
        }
    }

    impl TryFrom<proto::Socks5Local> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Socks5Local) -> Result<Self, Self::Error> {
            use crate::types::conversions::net::try_transport_protocol_from_i32;
            let remote_ip = value.remote_ip.parse::<Ipv4Addr>().map_err(|_| {
                FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (local) message from protobuf",
                )
            })?;
            Ok(AccessMethod::from(
                Socks5Local::new_with_transport_protocol(
                    (remote_ip, value.remote_port as u16),
                    value.local_port as u16,
                    try_transport_protocol_from_i32(value.remote_transport_protocol)?,
                ),
            ))
        }
    }

    impl TryFrom<proto::Socks5Remote> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Socks5Remote) -> Result<Self, Self::Error> {
            let proto::Socks5Remote { ip, port, auth } = value;
            let ip = ip.parse::<Ipv4Addr>().map_err(|_| {
                FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (remote) message from protobuf",
                )
            })?;
            let port = port as u16;

            Ok(AccessMethod::from(match auth.map(SocksAuth::from) {
                Some(auth) => Socks5Remote::new_with_authentication((ip, port), auth),
                None => Socks5Remote::new((ip, port)),
            }))
        }
    }

    impl TryFrom<proto::Shadowsocks> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Shadowsocks) -> Result<Self, Self::Error> {
            let ip = value.ip.parse::<Ipv4Addr>().map_err(|_| {
                FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (remote) message from protobuf",
                )
            })?;

            Ok(AccessMethod::from(Shadowsocks::new(
                (ip, value.port as u16),
                value.cipher,
                value.password,
            )))
        }
    }

    impl From<BuiltInAccessMethod> for proto::AccessMethod {
        fn from(value: BuiltInAccessMethod) -> Self {
            let access_method = match value {
                mullvad_types::access_method::BuiltInAccessMethod::Direct => {
                    proto::access_method::AccessMethod::Direct(proto::access_method::Direct {})
                }
                mullvad_types::access_method::BuiltInAccessMethod::Bridge => {
                    proto::access_method::AccessMethod::Bridges(proto::access_method::Bridges {})
                }
            };
            proto::AccessMethod {
                access_method: Some(access_method),
            }
        }
    }

    impl From<CustomProxy> for proto::AccessMethod {
        fn from(value: CustomProxy) -> Self {
            let access_method = match value {
                CustomProxy::Shadowsocks(ss) => {
                    proto::access_method::AccessMethod::Shadowsocks(proto::Shadowsocks {
                        ip: ss.endpoint.ip().to_string(),
                        port: u32::from(ss.endpoint.port()),
                        password: ss.password,
                        cipher: ss.cipher,
                    })
                }
                CustomProxy::Socks5Local(Socks5Local {
                    remote_endpoint,
                    local_port,
                }) => proto::access_method::AccessMethod::Socks5local(proto::Socks5Local {
                    remote_ip: remote_endpoint.address.ip().to_string(),
                    remote_port: remote_endpoint.address.port() as u32,
                    remote_transport_protocol: i32::from(proto::TransportProtocol::from(
                        remote_endpoint.protocol,
                    )),
                    local_port: u32::from(local_port),
                }),
                CustomProxy::Socks5Remote(Socks5Remote { endpoint, auth }) => {
                    proto::access_method::AccessMethod::Socks5remote(proto::Socks5Remote {
                        ip: endpoint.ip().to_string(),
                        port: u32::from(endpoint.port()),
                        auth: auth.map(proto::SocksAuth::from),
                    })
                }
            };

            proto::AccessMethod {
                access_method: Some(access_method),
            }
        }
    }

    impl From<Id> for proto::Uuid {
        fn from(value: Id) -> Self {
            proto::Uuid {
                value: value.to_string(),
            }
        }
    }

    impl TryFrom<proto::Uuid> for Id {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Uuid) -> Result<Self, Self::Error> {
            Self::from_string(value.value).ok_or(FromProtobufTypeError::InvalidArgument(
                "Could not parse UUID message from protobuf",
            ))
        }
    }

    impl TryFrom<&proto::AccessMethodSetting> for AccessMethodSetting {
        type Error = FromProtobufTypeError;

        fn try_from(value: &proto::AccessMethodSetting) -> Result<Self, Self::Error> {
            AccessMethodSetting::try_from(value.clone())
        }
    }
}
