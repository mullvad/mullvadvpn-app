use super::FromProtobufTypeError;
use crate::types::proto;

impl From<mullvad_types::api::RpcProxySettings> for proto::RpcProxySettings {
    fn from(settings: mullvad_types::api::RpcProxySettings) -> Self {
        match settings {
            mullvad_types::api::RpcProxySettings::Socks5(socks5) => proto::RpcProxySettings {
                r#type: Some(proto::rpc_proxy_settings::Type::Socks5(
                    proto::Socks5Settings {
                        proxy_endpoint: socks5.address.to_string(),
                    },
                )),
            },
        }
    }
}

impl TryFrom<proto::RpcProxySettings> for mullvad_types::api::RpcProxySettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::RpcProxySettings) -> Result<Self, Self::Error> {
        match settings.r#type {
            Some(proto::rpc_proxy_settings::Type::Socks5(socks5)) => Ok(
                mullvad_types::api::RpcProxySettings::Socks5(mullvad_types::api::Socks5Settings {
                    address: socks5
                        .proxy_endpoint
                        .parse()
                        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid endpoint"))?,
                }),
            ),
            _ => Err(FromProtobufTypeError::InvalidArgument("invalid proxy type")),
        }
    }
}
