use mullvad_api::{
    access_mode::AccessMethodResolver,
    proxy::{ApiConnectionMode, ProxyConfig},
    ApiEndpoint,
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_types::access_method::{AccessMethod, BuiltInAccessMethod};
use talpid_types::net::{
    proxy::CustomProxy, AllowedClients, AllowedEndpoint, Endpoint, TransportProtocol,
};
use tonic::async_trait;

use super::shadowsocks_loader::SwiftShadowsocksLoaderWrapperContext;

#[derive(Debug)]
pub struct SwiftAccessMethodResolver {
    endpoint: ApiEndpoint,
    domain: String,
    state: EncryptedDnsProxyState,
    bridge_provider: SwiftShadowsocksLoaderWrapperContext,
}

impl SwiftAccessMethodResolver {
    pub fn new(
        endpoint: ApiEndpoint,
        domain: String,
        state: EncryptedDnsProxyState,
        bridge_provider: SwiftShadowsocksLoaderWrapperContext,
    ) -> Self {
        Self {
            endpoint,
            domain,
            state,
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
                let bridge = self.bridge_provider.get_bridges()?;
                let proxy = CustomProxy::Shadowsocks(bridge);
                ApiConnectionMode::Proxied(ProxyConfig::from(proxy))
            }
            AccessMethod::BuiltIn(BuiltInAccessMethod::EncryptedDnsProxy) => {
                if let Err(error) = self.state.fetch_configs(self.domain.as_str()).await {
                    log::error!("{error:#?}");
                }
                let Some(edp) = self.state.next_configuration() else {
                    log::warn!("Could not select next Encrypted DNS proxy config");
                    return None;
                };
                ApiConnectionMode::Proxied(ProxyConfig::from(edp))
            }
            AccessMethod::Custom(config) => {
                ApiConnectionMode::Proxied(ProxyConfig::from(config.clone()))
            }
        };

        let allowed_endpoint = {
            let endpoint = connection_mode.get_endpoint().unwrap_or_else(|| {
                Endpoint::from_socket_address(
                    self.endpoint.address.unwrap(),
                    TransportProtocol::Tcp,
                )
            });
            let clients = AllowedClients::All;
            AllowedEndpoint { endpoint, clients }
        };

        Some((allowed_endpoint, connection_mode))
    }

    async fn default_connection_mode(&self) -> AllowedEndpoint {
        // TODO: Call the iOS Address cache implementation instead of returning the default endpoint
        let endpoint = ApiConnectionMode::Direct.get_endpoint().unwrap();
        AllowedEndpoint {
            endpoint,
            clients: AllowedClients::All,
        }
    }
}
