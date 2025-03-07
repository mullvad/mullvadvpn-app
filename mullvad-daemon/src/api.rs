use std::net::SocketAddr;

#[cfg(target_os = "android")]
use crate::DaemonCommand;
#[cfg(target_os = "android")]
use crate::DaemonEventSender;
use futures::{channel::mpsc, StreamExt};
use mullvad_api::AddressCache;
use mullvad_api::{
    access_mode::AccessMethodResolver,
    availability::ApiAvailability,
    proxy::{ApiConnectionMode, ProxyConfig},
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_management_interface::async_trait;
use mullvad_relay_selector::RelaySelector;
use mullvad_types::access_method::{AccessMethod, BuiltInAccessMethod};
#[cfg(target_os = "android")]
use talpid_core::mpsc::Sender;
use talpid_types::net::AllowedEndpoint;
use talpid_types::net::Endpoint;
use talpid_types::net::TransportProtocol;
use talpid_types::net::{proxy::CustomProxy, AllowedClients, Connectivity};

pub struct DaemonAccessMethodResolver {
    relay_selector: RelaySelector,
    encrypted_dns_proxy_cache: EncryptedDnsProxyState,
    address_cache: AddressCache,
}

impl DaemonAccessMethodResolver {
    pub fn new(
        relay_selector: RelaySelector,
        encrypted_dns_proxy_cache: EncryptedDnsProxyState,
        address_cache: AddressCache,
    ) -> Self {
        Self {
            relay_selector,
            encrypted_dns_proxy_cache,
            address_cache,
        }
    }
}

#[async_trait]
impl AccessMethodResolver for DaemonAccessMethodResolver {
    async fn resolve_access_method_setting(
        &mut self,
        access_method: &AccessMethod,
    ) -> Option<(AllowedEndpoint, ApiConnectionMode)> {
        let connection_mode = {
            match access_method {
                AccessMethod::BuiltIn(BuiltInAccessMethod::Direct) => ApiConnectionMode::Direct,
                AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge) => {
                    let Some(bridge) = self.relay_selector.get_bridge_forced() else {
                        log::warn!("Could not select a Mullvad bridge");
                        log::debug!("The relay list might be empty");
                        return None;
                    };
                    let proxy = CustomProxy::Shadowsocks(bridge);
                    ApiConnectionMode::Proxied(ProxyConfig::from(proxy))
                }
                AccessMethod::BuiltIn(BuiltInAccessMethod::EncryptedDnsProxy) => {
                    if let Err(error) = self
                        .encrypted_dns_proxy_cache
                        .fetch_configs("frakta.eu")
                        .await
                    {
                        log::warn!("Failed to fetch new Encrypted DNS Proxy configurations");
                        log::debug!("{error:#?}");
                    }
                    let Some(edp) = self.encrypted_dns_proxy_cache.next_configuration() else {
                        log::warn!("Could not select next Encrypted DNS proxy config");
                        return None;
                    };
                    ApiConnectionMode::Proxied(ProxyConfig::from(edp))
                }
                AccessMethod::Custom(config) => {
                    ApiConnectionMode::Proxied(ProxyConfig::from(config.clone()))
                }
            }
        };
        let endpoint =
            resolve_allowed_endpoint(&connection_mode, self.address_cache.get_address().await);
        Some((endpoint, connection_mode))
    }

    async fn default_connection_mode(&self) -> AllowedEndpoint {
        log::trace!("Defaulting to direct API connection");
        resolve_allowed_endpoint(
            &ApiConnectionMode::Direct,
            self.address_cache.get_address().await,
        )
    }
}

pub fn resolve_allowed_endpoint(
    connection_mode: &ApiConnectionMode,
    fallback: SocketAddr,
) -> AllowedEndpoint {
    let endpoint = match connection_mode.get_endpoint() {
        Some(endpoint) => endpoint,
        None => Endpoint::from_socket_address(fallback, TransportProtocol::Tcp),
    };
    let clients = allowed_clients(connection_mode);
    AllowedEndpoint { endpoint, clients }
}

#[cfg(unix)]
pub fn allowed_clients(connection_mode: &ApiConnectionMode) -> AllowedClients {
    match connection_mode {
        ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(_)) => AllowedClients::All,
        ApiConnectionMode::Direct | ApiConnectionMode::Proxied(_) => AllowedClients::Root,
    }
}

#[cfg(windows)]
pub fn allowed_clients(connection_mode: &ApiConnectionMode) -> AllowedClients {
    match connection_mode {
        ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(_)) => AllowedClients::all(),
        ApiConnectionMode::Direct | ApiConnectionMode::Proxied(_) => {
            let daemon_exe = std::env::current_exe().expect("failed to obtain executable path");
            vec![
                daemon_exe
                    .parent()
                    .expect("missing executable parent directory")
                    .join("mullvad-problem-report.exe"),
                daemon_exe,
            ]
            .into()
        }
    }
}

#[cfg(target_os = "android")]
pub(crate) fn create_bypass_tx(
    event_sender: &DaemonEventSender,
) -> Option<mpsc::Sender<mullvad_api::SocketBypassRequest>> {
    let (bypass_tx, mut bypass_rx) = mpsc::channel(1);
    let daemon_tx = event_sender.to_specialized_sender();
    tokio::spawn(async move {
        while let Some((raw_fd, done_tx)) = bypass_rx.next().await {
            if daemon_tx
                .send(DaemonCommand::BypassSocket(raw_fd, done_tx))
                .is_err()
            {
                log::error!("Can't send socket bypass request to daemon");
                break;
            }
        }
    });
    Some(bypass_tx)
}

/// Forwards the received values from `offline_state_rx` to the [`ApiAvailability`].
pub(crate) fn forward_offline_state(
    api_availability: ApiAvailability,
    mut offline_state_rx: mpsc::UnboundedReceiver<Connectivity>,
) {
    tokio::spawn(async move {
        let is_offline = offline_state_rx
            .next()
            .await
            .expect("missing initial offline state")
            .is_offline();
        log::info!(
            "Initial offline state - {state}",
            state = if is_offline { "offline" } else { "online" },
        );
        api_availability.set_offline(is_offline);

        while let Some(state) = offline_state_rx.next().await {
            log::info!("Detecting changes to offline state - {state:?}");
            api_availability.set_offline(state.is_offline());
        }
    });
}
