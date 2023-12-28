/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use talpid_types::net::proxy;

    impl From<proxy::CustomBridgeSettings> for proto::CustomProxySettings {
        fn from(settings: proxy::CustomBridgeSettings) -> Self {
            Self {
                custom_proxy: settings
                    .custom_bridge
                    .map(|custom_proxy| custom_proxy.into()),
            }
        }
    }

    impl TryFrom<proto::CustomProxySettings> for proxy::CustomBridgeSettings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::CustomProxySettings) -> Result<Self, Self::Error> {
            Ok(Self {
                custom_bridge: settings
                    .custom_proxy
                    .and_then(|custom_proxy| proxy::CustomProxy::try_from(custom_proxy).ok()),
            })
        }
    }
}
