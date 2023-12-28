/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use talpid_types::net::proxy;

    impl From<proxy::CustomBridgeSettings> for proto::CustomBridgeSettings {
        fn from(settings: proxy::CustomBridgeSettings) -> Self {
            Self {
                custom_bridge: settings
                    .custom_bridge
                    .map(|custom_proxy| custom_proxy.into()),
            }
        }
    }

    impl TryFrom<proto::CustomBridgeSettings> for proxy::CustomBridgeSettings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::CustomBridgeSettings) -> Result<Self, Self::Error> {
            Ok(Self {
                custom_bridge: settings
                    .custom_bridge
                    .and_then(|custom_proxy| proxy::CustomProxy::try_from(custom_proxy).ok()),
            })
        }
    }
}
