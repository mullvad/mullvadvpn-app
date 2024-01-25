/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::access_method;

    impl From<access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: access_method::Settings) -> Self {
            Self {
                direct: Some(settings.direct().clone().into()),
                mullvad_bridges: Some(settings.mullvad_bridges().clone().into()),
                access_method_settings: settings
                    .iter_custom()
                    .cloned()
                    .map(|method| method.into())
                    .collect(),
            }
        }
    }

    impl TryFrom<proto::ApiAccessMethodSettings> for access_method::Settings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::ApiAccessMethodSettings) -> Result<Self, Self::Error> {
            let direct = settings
                .direct
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Direct Access Method from protobuf",
                ))
                .and_then(access_method::AccessMethodSetting::try_from)?;

            let mullvad_bridges = settings
                .mullvad_bridges
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "Could not deserialize Mullvad Bridges Access Method from protobuf",
                ))
                .and_then(access_method::AccessMethodSetting::try_from)?;

            let user_defined = settings
                .access_method_settings
                .iter()
                .map(access_method::AccessMethodSetting::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(access_method::Settings::new(
                direct,
                mullvad_bridges,
                user_defined,
            ))
        }
    }
}

/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethod`] type to the internal
/// [`mullvad_types::access_method::AccessMethodSetting`] data type.
mod data {
    use crate::types::{proto, FromProtobufTypeError};
    use mullvad_types::access_method::{
        AccessMethod, AccessMethodSetting, BuiltInAccessMethod, Id,
    };
    use talpid_types::net::proxy::{CustomProxy, Shadowsocks, Socks5Local, Socks5Remote};

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
                proto::access_method::AccessMethod::Custom(custom) => {
                    CustomProxy::try_from(custom).map(AccessMethod::from)?
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
            Socks5Local::try_from(value).map(AccessMethod::from)
        }
    }

    impl TryFrom<proto::Socks5Remote> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Socks5Remote) -> Result<Self, Self::Error> {
            Socks5Remote::try_from(value).map(AccessMethod::from)
        }
    }

    impl TryFrom<proto::Shadowsocks> for AccessMethod {
        type Error = FromProtobufTypeError;

        fn try_from(value: proto::Shadowsocks) -> Result<Self, Self::Error> {
            Shadowsocks::try_from(value).map(AccessMethod::from)
        }
    }

    impl From<BuiltInAccessMethod> for proto::AccessMethod {
        fn from(value: BuiltInAccessMethod) -> Self {
            proto::AccessMethod {
                access_method: Some(proto::access_method::AccessMethod::from(value)),
            }
        }
    }

    impl From<BuiltInAccessMethod> for proto::access_method::AccessMethod {
        fn from(value: BuiltInAccessMethod) -> Self {
            match value {
                mullvad_types::access_method::BuiltInAccessMethod::Direct => {
                    proto::access_method::AccessMethod::Direct(proto::access_method::Direct {})
                }
                mullvad_types::access_method::BuiltInAccessMethod::Bridge => {
                    proto::access_method::AccessMethod::Bridges(proto::access_method::Bridges {})
                }
            }
        }
    }

    impl From<CustomProxy> for proto::AccessMethod {
        fn from(value: CustomProxy) -> Self {
            proto::AccessMethod {
                access_method: Some(proto::access_method::AccessMethod::from(value)),
            }
        }
    }

    impl From<CustomProxy> for proto::access_method::AccessMethod {
        fn from(value: CustomProxy) -> Self {
            proto::access_method::AccessMethod::Custom(proto::CustomProxy::from(value))
        }
    }

    impl From<&Id> for proto::Uuid {
        fn from(value: &Id) -> Self {
            proto::Uuid {
                value: value.to_string(),
            }
        }
    }

    impl From<Id> for proto::Uuid {
        fn from(value: Id) -> Self {
            proto::Uuid::from(&value)
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
