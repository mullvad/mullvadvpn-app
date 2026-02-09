use mullvad_api::{
    AddressCache, AddressCacheBacking, AddressCacheError, ApiEndpoint, access_mode::AccessMethodResolver, proxy::{ApiConnectionMode, ProxyConfig}
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_types::access_method::{AccessMethod, BuiltInAccessMethod};
use talpid_types::net::{
    AllowedClients, AllowedEndpoint, Endpoint, TransportProtocol, proxy::CustomProxy,
};
use tonic::async_trait;

use crate::get_string;

use super::{
    late_string_deallocator::LateStringDeallocator,
    shadowsocks_loader::SwiftShadowsocksLoaderWrapper,
};

unsafe extern "C" {
    pub fn swift_store_address_cache(data: *const u8, data_size: u64);

    pub fn swift_read_address_cache() -> LateStringDeallocator;
}

#[derive(Clone)]
pub struct IOSAddressCacheBacking {}

#[async_trait]
impl AddressCacheBacking for IOSAddressCacheBacking {
    async fn read(&self) -> Result<Vec<u8>, AddressCacheError> {
        let lsd = unsafe { swift_read_address_cache() };
        let val = unsafe { get_string(lsd.ptr) };
        Ok(val.as_bytes().to_vec())
    }

    async fn write(&self, data: &[u8]) -> Result<(), AddressCacheError> {
        unsafe { swift_store_address_cache(data.as_ptr(), data.len().try_into().unwrap()) };
        Ok(())
    }
}

#[derive(Debug)]
pub struct SwiftAccessMethodResolver {
    endpoint: ApiEndpoint,
    domain: String,
    state: EncryptedDnsProxyState,
    bridge_provider: SwiftShadowsocksLoaderWrapper,
    address_cache: AddressCache,
}

impl SwiftAccessMethodResolver {
    pub fn new(
        endpoint: ApiEndpoint,
        domain: String,
        state: EncryptedDnsProxyState,
        bridge_provider: SwiftShadowsocksLoaderWrapper,
        address_cache: AddressCache,
    ) -> Self {
        Self {
            endpoint,
            domain,
            state,
            bridge_provider,
            address_cache,
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
        let endpoint =
            Endpoint::from_socket_address(self.address_cache.get_address().await, TransportProtocol::Tcp);

        AllowedEndpoint {
            endpoint,
            clients: AllowedClients::All,
        }
    }
}
