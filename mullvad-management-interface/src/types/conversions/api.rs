use super::FromProtobufTypeError;
use crate::types::proto;

impl From<mullvad_types::api::RpcProxySettings> for proto::RpcProxySettings {
    fn from(settings: mullvad_types::api::RpcProxySettings) -> Self {
        match settings {
            mullvad_types::api::RpcProxySettings::LocalSocks5Settings(port) => {
                proto::RpcProxySettings {
                    r#type: Some(proto::rpc_proxy_settings::Type::LocalSocks5(
                        proto::LocalSocks5Settings {
                            port: u32::from(port.port),
                        },
                    )),
                }
            }
        }
    }
}

impl TryFrom<proto::RpcProxySettings> for mullvad_types::api::RpcProxySettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::RpcProxySettings) -> Result<Self, Self::Error> {
        match settings.r#type {
            Some(proto::rpc_proxy_settings::Type::LocalSocks5(socks5)) => {
                Ok(mullvad_types::api::RpcProxySettings::LocalSocks5Settings(
                    mullvad_types::api::LocalSocks5Settings {
                        port: u16::try_from(socks5.port)
                            .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid port"))?,
                    },
                ))
            }
            _ => Err(FromProtobufTypeError::InvalidArgument("invalid proxy type")),
        }
    }
}
