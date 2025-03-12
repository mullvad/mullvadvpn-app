#[cfg(target_os = "android")]
use crate::DaemonCommand;
#[cfg(target_os = "android")]
use crate::DaemonEventSender;
use futures::{channel::mpsc, StreamExt};
use mullvad_api::{
    access_mode::BridgeAndDNSProxy,
    availability::ApiAvailability,
    proxy::{AllowedClientsProvider, ApiConnectionMode, ProxyConfig},
};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_management_interface::async_trait;
use mullvad_types::{
    access_method::{AccessMethod, AccessMethodSetting, BuiltInAccessMethod},
    relay_list::ShadowsocksBridgeProvider,
};
#[cfg(target_os = "android")]
use talpid_core::mpsc::Sender;
use talpid_types::net::{proxy::CustomProxy, AllowedClients, Connectivity};

pub struct BridgeAndDNSProxyProvider<T> {
    bridge_provider: T,
    encrypted_dns_proxy_cache: EncryptedDnsProxyState,
}

impl<T: ShadowsocksBridgeProvider> BridgeAndDNSProxyProvider<T> {
    pub fn new(bridge_provider: T, encrypted_dns_proxy_cache: EncryptedDnsProxyState) -> Self {
        Self {
            bridge_provider,
            encrypted_dns_proxy_cache,
        }
    }
}

#[async_trait]
impl<T> BridgeAndDNSProxy for BridgeAndDNSProxyProvider<T>
where
    T: ShadowsocksBridgeProvider,
{
    async fn match_access_method(
        &mut self,
        access_method: &AccessMethodSetting,
    ) -> Option<ApiConnectionMode> {
        let connection_mode = {
            match &access_method.access_method {
                AccessMethod::BuiltIn(BuiltInAccessMethod::Direct) => ApiConnectionMode::Direct,
                AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge) => {
                    let Some(bridge) = self.bridge_provider.get_bridge_forced() else {
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
        Some(connection_mode)
    }
}

#[derive(Clone, Copy)]
pub struct AllowedClientsSelector {}

impl AllowedClientsProvider for AllowedClientsSelector {
    #[cfg(unix)]
    fn allowed_clients(connection_mode: &ApiConnectionMode) -> AllowedClients {
        match connection_mode {
            ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(_)) => AllowedClients::All,
            ApiConnectionMode::Direct | ApiConnectionMode::Proxied(_) => AllowedClients::Root,
        }
    }

    #[cfg(windows)]
    fn allowed_clients(connection_mode: &ApiConnectionMode) -> AllowedClients {
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
