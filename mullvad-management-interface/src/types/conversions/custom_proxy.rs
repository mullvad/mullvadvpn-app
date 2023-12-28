/// Implements conversions for the auxilliary
/// [`crate::types::proto::ApiAccessMethodSettings`] type to the internal
/// [`mullvad_types::access_method::Settings`] data type.
mod settings {
    use crate::types::{proto, FromProtobufTypeError};
    use talpid_types::net::proxy;

    impl From<proxy::CustomProxySettings> for proto::CustomProxySettings {
        fn from(settings: proxy::CustomProxySettings) -> Self {
            Self {
                custom_proxy: settings
                    .custom_proxy
                    .map(|custom_proxy| custom_proxy.into()),
                active: settings.active,
            }
        }
    }

    impl TryFrom<proto::CustomProxySettings> for proxy::CustomProxySettings {
        type Error = FromProtobufTypeError;

        fn try_from(settings: proto::CustomProxySettings) -> Result<Self, Self::Error> {
            Ok(Self {
                custom_proxy: settings
                    .custom_proxy
                    .and_then(|custom_proxy| proxy::CustomProxy::try_from(custom_proxy).ok()),
                active: settings.active,
            })
        }
    }
}
