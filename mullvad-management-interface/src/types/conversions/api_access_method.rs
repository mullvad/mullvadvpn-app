/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
pub mod settings {
    use crate::types::{
        proto, rpc::api_access_method_update::ApiAccessMethodUpdate, FromProtobufTypeError,
    };
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

    impl From<ApiAccessMethodUpdate> for proto::ApiAccessMethodUpdate {
        fn from(value: ApiAccessMethodUpdate) -> Self {
            proto::ApiAccessMethodUpdate {
                id: Some(proto::Uuid::from(value.id)),
                access_method: Some(proto::ApiAccessMethod::from(value.access_method)),
            }
        }
    }

    impl TryFrom<proto::ApiAccessMethodUpdate> for ApiAccessMethodUpdate {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethodUpdate) -> Result<Self, Self::Error> {
            let api_access_method = value
                .access_method
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not convert Access Method from protobuf",
                ))
                .and_then(api_access::AccessMethodSetting::try_from)?;

            let id = value
                .id
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not convert Access Method from protobuf",
                ))
                .map(api_access::ApiAccessMethodId::from)?;

            Ok(ApiAccessMethodUpdate {
                id,
                access_method: api_access_method,
            })
        }
    }
}

/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethod`] type to the internal
/// [`mullvad_types::access_method::AccessMethod`] data type.
mod data {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::api_access::{
        AccessMethod, AccessMethodSetting, ApiAccessMethodId, BuiltInAccessMethod,
        CustomAccessMethod, Shadowsocks, Socks5, Socks5Local, Socks5Remote,
    };

    impl TryFrom<proto::ApiAccessMethod> for AccessMethodSetting {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            let id: ApiAccessMethodId = value
                .id
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))?
                .into();
            let name = value.name;
            let enabled = value.enabled;
            let access_method = value
                .access_method
                .and_then(|access_method_field| access_method_field.access_method)
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Access Method from protobuf",
                ))?;

            let access_method = match access_method {
                proto::access_method::AccessMethod::Direct(proto::access_method::Direct {}) => {
                    AccessMethod::from(BuiltInAccessMethod::Direct)
                }

                proto::access_method::AccessMethod::Bridges(proto::access_method::Bridges {}) => {
                    AccessMethod::from(BuiltInAccessMethod::Bridge)
                }
                proto::access_method::AccessMethod::Socks5local(local) => {
                    let socks = Socks5Local::from_args(
                        local.ip,
                        local.port as u16,
                        local.local_port as u16,
                    )
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Could not parse Socks5 (local) message from protobuf",
                    ))?;
                    AccessMethod::from(socks)
                }

                proto::access_method::AccessMethod::Socks5remote(remote) => {
                    let socks = Socks5Remote::from_args(remote.ip, remote.port as u16).ok_or({
                        FromProtobufTypeError::InvalidArgument(
                            "Could not parse Socks5 (remote) message from protobuf",
                        )
                    })?;
                    AccessMethod::from(socks)
                }
                proto::access_method::AccessMethod::Shadowsocks(ss) => {
                    let socks =
                        Shadowsocks::from_args(ss.ip, ss.port as u16, ss.cipher, ss.password)
                            .ok_or(FromProtobufTypeError::InvalidArgument(
                                "Could not parse Shadowsocks message from protobuf",
                            ))?;
                    AccessMethod::from(socks)
                }
            };

            Ok(AccessMethodSetting::with_id(
                id,
                name,
                enabled,
                access_method,
            ))
        }
    }

    impl From<ApiAccessMethodId> for proto::Uuid {
        fn from(value: ApiAccessMethodId) -> Self {
            proto::Uuid {
                value: value.to_string(),
            }
        }
    }

    impl From<proto::Uuid> for ApiAccessMethodId {
        fn from(value: proto::Uuid) -> Self {
            Self::from_string(value.value)
        }
    }

    impl From<AccessMethodSetting> for proto::ApiAccessMethod {
        fn from(value: AccessMethodSetting) -> Self {
            let id = proto::Uuid::from(value.get_id());
            let name = value.get_name();
            let enabled = value.enabled();
            let access_method: proto::access_method::AccessMethod = match value.access_method {
                AccessMethod::Custom(value) => match value {
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
                },
                AccessMethod::BuiltIn(value) => match value {
                    mullvad_types::api_access::BuiltInAccessMethod::Direct => {
                        proto::access_method::AccessMethod::Direct(proto::access_method::Direct {})
                    }
                    mullvad_types::api_access::BuiltInAccessMethod::Bridge => {
                        proto::access_method::AccessMethod::Bridges(
                            proto::access_method::Bridges {},
                        )
                    }
                },
            };

            proto::ApiAccessMethod {
                id: Some(id),
                name,
                enabled,
                access_method: Some(proto::AccessMethod {
                    access_method: Some(access_method),
                }),
            }
        }
    }

    impl TryFrom<&proto::ApiAccessMethod> for AccessMethodSetting {
        type Error = FromProtobufTypeError;

        fn try_from(value: &proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            AccessMethodSetting::try_from(value.clone())
        }
    }
}
