#[cfg(target_os = "android")]
use crate::DaemonCommand;
#[cfg(target_os = "android")]
use crate::DaemonEventSender;
use futures::channel::mpsc;
use futures::StreamExt;
use mullvad_api::availability::ApiAvailability;
use mullvad_api::proxy::AllowedClientsProvider;
use mullvad_api::proxy::ApiConnectionMode;
use mullvad_api::proxy::ProxyConfig;
#[cfg(target_os = "android")]
use talpid_core::mpsc::Sender;
use talpid_types::net::AllowedClients;
use talpid_types::net::Connectivity;

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

#[derive(Clone, Copy)]
pub struct AllowedClientsSelector {}

impl AllowedClientsProvider for AllowedClientsSelector {
    #[cfg(unix)]
    fn allowed_clients(&self, connection_mode: &ApiConnectionMode) -> AllowedClients {
        match connection_mode {
            ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(_)) => AllowedClients::All,
            ApiConnectionMode::Direct | ApiConnectionMode::Proxied(_) => AllowedClients::Root,
        }
    }

    #[cfg(windows)]
    fn allowed_clients(&self, connection_mode: &ApiConnectionMode) -> AllowedClients {
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
