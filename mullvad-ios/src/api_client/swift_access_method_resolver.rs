use mullvad_api::{
    access_mode::AccessMethodResolver,
    proxy::{ApiConnectionMode, ProxyConfig},
};
use mullvad_types::access_method::{AccessMethod, BuiltInAccessMethod};
use talpid_types::{
    self,
    net::{
        proxy::{CustomProxy, Shadowsocks},
        AllowedClients, AllowedEndpoint,
    },
};
use tonic::async_trait;

use super::swift_shadowsocks_loader::SwiftShadowsocksLoaderWrapperContext;

#[derive(Debug)]
pub struct SwiftAccessMethodResolver {
    bridge_provider: SwiftShadowsocksLoaderWrapperContext,
}

impl SwiftAccessMethodResolver {
    pub fn new(bridge_provider: SwiftShadowsocksLoaderWrapperContext) -> Self {
        Self { bridge_provider }
    }
}

#[async_trait]
impl AccessMethodResolver for SwiftAccessMethodResolver {
    async fn resolve_access_method_setting(
        &mut self,
        access_method: &AccessMethod,
    ) -> Option<(AllowedEndpoint, ApiConnectionMode)> {
        let connection_mode = match access_method {
            AccessMethod::BuiltIn(BuiltInAccessMethod::Direct) => ApiConnectionMode::Direct,
            // TODO: This should call upon the relay selector and get bridges
            AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge) => {
                let Some(bridge) = self.bridge_provider.get_bridges() else {
                    return None;
                };
                let proxy = CustomProxy::Shadowsocks(bridge);
                ApiConnectionMode::Proxied(ProxyConfig::from(proxy))
            }
            AccessMethod::BuiltIn(BuiltInAccessMethod::EncryptedDnsProxy) => {
                ApiConnectionMode::Direct
            }
            AccessMethod::Custom(config) => {
                ApiConnectionMode::Proxied(ProxyConfig::from(config.clone()))
            }
        };

        Some((
            AllowedEndpoint {
                endpoint: connection_mode.get_endpoint().unwrap(),
                clients: AllowedClients::All,
            },
            connection_mode,
        ))
    }

    async fn default_connection_mode(&self) -> AllowedEndpoint {
        // TODO: What should happen here, should we call the address cache ?
        // TODO: Check the settings to see which access method should be used
        let endpoint = ApiConnectionMode::Direct.get_endpoint().unwrap();
        return AllowedEndpoint {
            endpoint,
            clients: AllowedClients::All,
        };
    }
}
