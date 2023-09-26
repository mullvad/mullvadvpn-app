/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::api_access;

    impl From<&api_access::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: &api_access::Settings) -> Self {
            Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(|method| method.clone().into())
                    .collect(),
            }
        }
    }

    impl From<api_access::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: api_access::Settings) -> Self {
            proto::ApiAccessMethodSettings::from(&settings)
        }
    }

    impl TryFrom<proto::ApiAccessMethodSettings> for api_access::Settings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::ApiAccessMethodSettings) -> Result<Self, Self::Error> {
            Ok(Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(api_access::AccessMethodSetting::try_from)
                    .collect::<Result<Vec<api_access::AccessMethodSetting>, _>>()?,
            })
        }
    }
}

/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethod`] type to the internal
/// [`mullvad_types::access_method::AccessMethodSetting`] data type.
mod data {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::api_access::{
        AccessMethod, AccessMethodSetting, ApiAccessMethodId, BuiltInAccessMethod,
        CustomAccessMethod, Shadowsocks, Socks5, Socks5Local, Socks5Remote,
    };

    impl TryFrom<proto::ApiAccessMethod> for AccessMethodSetting {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            let id = value
                .id
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))
                .and_then(ApiAccessMethodId::try_from)?;
            let name = value.name;
            let enabled = value.enabled;
            let access_method = value
                .access_method
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))
                .and_then(|access_method_field| AccessMethod::try_from(access_method_field))?;

            Ok(AccessMethodSetting::with_id(
                id,
                name,
                enabled,
                access_method,
            ))
        }
    }

    impl From<AccessMethodSetting> for proto::ApiAccessMethod {
        fn from(value: AccessMethodSetting) -> Self {
            let id = proto::Uuid::from(value.get_id());
            let name = value.get_name();
            let enabled = value.enabled();
            proto::ApiAccessMethod {
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

    impl TryFrom<proto::access_method::Socks5Local> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::access_method::Socks5Local) -> Result<Self, Self::Error> {
            Socks5Local::from_args(value.ip, value.port as u16, value.local_port as u16)
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not parse Socks5 (local) message from protobuf",
                ))
                .map(AccessMethod::from)
        }
    }

    impl TryFrom<proto::access_method::Socks5Remote> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::access_method::Socks5Remote) -> Result<Self, Self::Error> {
            Socks5Remote::from_args(value.ip, value.port as u16)
                .ok_or({
                    FromProtobufTypeError::InvalidArgument(
                        "Could not parse Socks5 (remote) message from protobuf",
                    )
                })
                .map(AccessMethod::from)
        }
    }

    impl TryFrom<proto::access_method::Shadowsocks> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::access_method::Shadowsocks) -> Result<Self, Self::Error> {
            Shadowsocks::from_args(value.ip, value.port as u16, value.cipher, value.password)
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not parse Shadowsocks message from protobuf",
                ))
                .map(AccessMethod::from)
        }
    }

    impl From<BuiltInAccessMethod> for proto::AccessMethod {
        fn from(value: BuiltInAccessMethod) -> Self {
            let access_method = match value {
                mullvad_types::api_access::BuiltInAccessMethod::Direct => {
                    proto::access_method::AccessMethod::Direct(proto::access_method::Direct {})
                }
                mullvad_types::api_access::BuiltInAccessMethod::Bridge => {
                    proto::access_method::AccessMethod::Bridges(proto::access_method::Bridges {})
                }
            };
            proto::AccessMethod {
                access_method: Some(access_method),
            }
        }
    }

    impl From<CustomAccessMethod> for proto::AccessMethod {
        fn from(value: CustomAccessMethod) -> Self {
            let access_method = match value {
                CustomAccessMethod::Shadowsocks(ss) => {
                    proto::access_method::AccessMethod::Shadowsocks(
                        proto::access_method::Shadowsocks {
                            ip: ss.peer.ip().to_string(),
                            port: ss.peer.port() as u32,
                            password: ss.password,
                            cipher: ss.cipher,
                        },
                    )
                }
                CustomAccessMethod::Socks5(Socks5::Local(Socks5Local { peer, port })) => {
                    proto::access_method::AccessMethod::Socks5local(
                        proto::access_method::Socks5Local {
                            ip: peer.ip().to_string(),
                            port: peer.port() as u32,
                            local_port: port as u32,
                        },
                    )
                }
                CustomAccessMethod::Socks5(Socks5::Remote(Socks5Remote { peer })) => {
                    proto::access_method::AccessMethod::Socks5remote(
                        proto::access_method::Socks5Remote {
                            ip: peer.ip().to_string(),
                            port: peer.port() as u32,
                        },
                    )
                }
            };

            proto::AccessMethod {
                access_method: Some(access_method),
            }
        }
    }

    impl From<ApiAccessMethodId> for proto::Uuid {
        fn from(value: ApiAccessMethodId) -> Self {
            proto::Uuid {
                value: value.to_string(),
            }
        }
    }

    impl TryFrom<proto::Uuid> for ApiAccessMethodId {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Uuid) -> Result<Self, Self::Error> {
            Self::from_string(value.value).ok_or(FromProtobufTypeError::InvalidArgument(
                "Could not parse UUID message from protobuf",
            ))
        }
    }

    impl TryFrom<&proto::ApiAccessMethod> for AccessMethodSetting {
        type Error = FromProtobufTypeError;

        fn try_from(value: &proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            AccessMethodSetting::try_from(value.clone())
        }
    }
}
