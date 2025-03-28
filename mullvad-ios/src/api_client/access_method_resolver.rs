use mullvad_api::{
    access_mode::AccessMethodResolver,
    proxy::{ApiConnectionMode, ProxyConfig},
    ApiEndpoint,
};
use mullvad_types::access_method::{AccessMethod, BuiltInAccessMethod};
use talpid_types::net::{
    proxy::CustomProxy, AllowedClients, AllowedEndpoint, Endpoint, TransportProtocol,
};
use tonic::async_trait;

use super::shadowsocks_loader::SwiftShadowsocksLoaderWrapperContext;

#[derive(Debug)]
pub struct SwiftAccessMethodResolver {
    endpoint: ApiEndpoint,
    bridge_provider: SwiftShadowsocksLoaderWrapperContext,
}

impl SwiftAccessMethodResolver {
    pub fn new(
        endpoint: ApiEndpoint,
        bridge_provider: SwiftShadowsocksLoaderWrapperContext,
    ) -> Self {
        Self {
            endpoint,
            bridge_provider,
        }
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
            AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge) => {
                let Some(bridge) = self.bridge_provider.get_bridges() else {
                    return None;
                };
                let proxy = CustomProxy::Shadowsocks(bridge);
                ApiConnectionMode::Proxied(ProxyConfig::from(proxy))
            }
            // TODO: Reuse the eDNS proxy from encrypted_dns_proxy.rs ?
            AccessMethod::BuiltIn(BuiltInAccessMethod::EncryptedDnsProxy) => {
                ApiConnectionMode::Direct
            }
            AccessMethod::Custom(config) => {
                ApiConnectionMode::Proxied(ProxyConfig::from(config.clone()))
            }
        };

        Some((
            AllowedEndpoint {
                endpoint: match connection_mode.get_endpoint() {
                    Some(endpoint) => endpoint,
                    None => Endpoint::from_socket_address(
                        self.endpoint.address.unwrap(),
                        TransportProtocol::Tcp,
                    ),
                },
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
