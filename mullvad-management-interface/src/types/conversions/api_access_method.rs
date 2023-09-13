/// Implements conversions for the auxilliary proto AccessMethod type to the internal AccessMethod data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::api_access_method;

    impl From<&api_access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: &api_access_method::Settings) -> Self {
            Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(|method| method.clone().into())
                    .collect(),
            }
        }
    }

    impl From<api_access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: api_access_method::Settings) -> Self {
            proto::ApiAccessMethodSettings::from(&settings)
        }
    }

    impl TryFrom<proto::ApiAccessMethodSettings> for api_access_method::Settings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::ApiAccessMethodSettings) -> Result<Self, Self::Error> {
            Ok(Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(api_access_method::AccessMethod::try_from)
                    .collect::<Result<Vec<api_access_method::AccessMethod>, _>>()?,
            })
        }
    }

    impl From<api_access_method::daemon::ApiAccessMethodReplace> for proto::ApiAccessMethodReplace {
        fn from(value: api_access_method::daemon::ApiAccessMethodReplace) -> Self {
            proto::ApiAccessMethodReplace {
                index: value.index as u32,
                access_method: Some(value.access_method.into()),
            }
        }
    }

    impl TryFrom<proto::ApiAccessMethodReplace> for api_access_method::daemon::ApiAccessMethodReplace {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethodReplace) -> Result<Self, Self::Error> {
            Ok(api_access_method::daemon::ApiAccessMethodReplace {
                index: value.index as usize,
                access_method: value
                    .access_method
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Could not convert Access Method from protobuf",
                    ))
                    .and_then(TryInto::try_into)?,
            })
        }
    }
}

/// Implements conversions for the 'main' AccessMethod data type.
mod data {
    use crate::types::{
        proto::{self, api_access_method::socks5::Socks5type},
        FromProtobufTypeError,
    };
    use mullvad_types::api_access_method::{
        AccessMethod, BuiltInAccessMethod, ObfuscationProtocol, Shadowsocks, Socks5, Socks5Local,
        Socks5Remote,
    };

    impl TryFrom<proto::ApiAccessMethods> for Vec<AccessMethod> {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethods) -> Result<Self, Self::Error> {
            value
                .api_access_methods
                .iter()
                .map(AccessMethod::try_from)
                .collect()
        }
    }

    impl TryFrom<proto::ApiAccessMethod> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            let access_method =
                value
                    .access_method
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Could not convert Access Method from protobuf",
                    ))?;
            Ok(match access_method {
                proto::api_access_method::AccessMethod::Socks5(socks) => {
                    match socks.socks5type.unwrap() {
                        Socks5type::Local(local) => Socks5Local::from_args(
                            local.ip,
                            local.port as u16,
                            local.local_port as u16,
                        )
                        .ok_or(FromProtobufTypeError::InvalidArgument(
                            "Could not parse Socks5 (local) message from protobuf",
                        ))?
                        .into(),

                        Socks5type::Remote(remote) => {
                            Socks5Remote::from_args(remote.ip, remote.port as u16)
                                .ok_or({
                                    FromProtobufTypeError::InvalidArgument(
                                        "Could not parse Socks5 (remote) message from protobuf",
                                    )
                                })?
                                .into()
                        }
                    }
                }
                proto::api_access_method::AccessMethod::Shadowsocks(ss) => {
                    Shadowsocks::from_args(ss.ip, ss.port as u16, ss.cipher, ss.password)
                        .ok_or(FromProtobufTypeError::InvalidArgument(
                            "Could not parse Shadowsocks message from protobuf",
                        ))?
                        .into()
                }
                proto::api_access_method::AccessMethod::Direct(_) => {
                    BuiltInAccessMethod::Direct.into()
                }
                proto::api_access_method::AccessMethod::Bridges(_) => {
                    BuiltInAccessMethod::Bridge.into()
                }
            })
        }
    }

    impl From<AccessMethod> for proto::ApiAccessMethod {
        fn from(value: AccessMethod) -> Self {
            match value {
                AccessMethod::Custom(value) => match value.access_method {
                    ObfuscationProtocol::Shadowsocks(ss) => proto::api_access_method::Shadowsocks {
                        id: value.id,
                        ip: ss.peer.ip().to_string(),
                        port: ss.peer.port() as u32,
                        password: ss.password,
                        cipher: ss.cipher,
                    }
                    .into(),

                    ObfuscationProtocol::Socks5(Socks5::Local(Socks5Local { peer, port })) => {
                        proto::api_access_method::Socks5Local {
                            id: value.id,
                            ip: peer.ip().to_string(),
                            port: peer.port() as u32,
                            local_port: port as u32,
                        }
                        .into()
                    }
                    ObfuscationProtocol::Socks5(Socks5::Remote(Socks5Remote { peer })) => {
                        proto::api_access_method::Socks5Remote {
                            id: value.id,
                            ip: peer.ip().to_string(),
                            port: peer.port() as u32,
                        }
                        .into()
                    }
                },
                AccessMethod::BuiltIn(value) => match value {
                    mullvad_types::api_access_method::BuiltInAccessMethod::Direct => {
                        proto::api_access_method::Direct {}.into()
                    }
                    mullvad_types::api_access_method::BuiltInAccessMethod::Bridge => {
                        proto::api_access_method::Bridges {}.into()
                    }
                },
            }
        }
    }

    impl TryFrom<&proto::ApiAccessMethod> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: &proto::ApiAccessMethod) -> Result<Self, Self::Error> {
            AccessMethod::try_from(value.clone())
        }
    }

    impl From<Vec<AccessMethod>> for proto::ApiAccessMethods {
        fn from(value: Vec<AccessMethod>) -> proto::ApiAccessMethods {
            proto::ApiAccessMethods {
                api_access_methods: value.iter().map(|method| method.clone().into()).collect(),
            }
        }
    }

    impl From<proto::api_access_method::Shadowsocks> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Shadowsocks) -> Self {
            proto::api_access_method::AccessMethod::Shadowsocks(value).into()
        }
    }

    impl From<proto::api_access_method::Socks5> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Socks5) -> Self {
            proto::api_access_method::AccessMethod::Socks5(value).into()
        }
    }

    impl From<proto::api_access_method::socks5::Socks5type> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::socks5::Socks5type) -> Self {
            proto::api_access_method::AccessMethod::Socks5(proto::api_access_method::Socks5 {
                socks5type: Some(value),
            })
            .into()
        }
    }

    impl From<proto::api_access_method::Socks5Local> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Socks5Local) -> Self {
            proto::api_access_method::socks5::Socks5type::Local(value).into()
        }
    }

    impl From<proto::api_access_method::Socks5Remote> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Socks5Remote) -> Self {
            proto::api_access_method::socks5::Socks5type::Remote(value).into()
        }
    }

    impl From<proto::api_access_method::Direct> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Direct) -> Self {
            proto::api_access_method::AccessMethod::Direct(value).into()
        }
    }

    impl From<proto::api_access_method::Bridges> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Bridges) -> Self {
            proto::api_access_method::AccessMethod::Bridges(value).into()
        }
    }

    impl From<proto::api_access_method::AccessMethod> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::AccessMethod) -> Self {
            proto::ApiAccessMethod {
                access_method: Some(value),
            }
        }
    }
}
