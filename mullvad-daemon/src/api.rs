#[cfg(target_os = "android")]
use crate::{DaemonCommand, DaemonEventSender};
use futures::{
    channel::{mpsc, oneshot},
    Future, Stream, StreamExt,
};
use mullvad_api::{
    availability::ApiAvailabilityHandle,
    proxy::{ApiConnectionMode, ProxyConfig},
    ApiEndpointUpdateCallback,
};
use mullvad_relay_selector::RelaySelector;
use mullvad_types::api_access_method;
use std::{
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex, Weak},
    task::Poll,
};
#[cfg(target_os = "android")]
use talpid_core::mpsc::Sender;
use talpid_core::tunnel_state_machine::TunnelCommand;
use talpid_types::{
    net::{openvpn::ProxySettings, AllowedEndpoint, Endpoint, TransportProtocol},
    ErrorExt,
};

/// TODO: Update this comment
/// A stream that returns the next API connection mode to use for reaching the API.
///
/// When `mullvad-api` fails to contact the API, it requests a new connection mode.
/// The API can be connected to either directly (i.e., [`ApiConnectionMode::Direct`])
/// via a bridge ([`ApiConnectionMode::Proxied`]) or via any supported obfuscation protocol ([`ObfuscationProtocol`]).
///
/// * Every 3rd attempt returns [`ApiConnectionMode::Direct`].
/// * Any other attempt returns a configuration for the bridge that is closest to the selected relay
///   location and matches all bridge constraints.
/// * When no matching bridge is found, e.g. if the selected hosting providers don't match any
///   bridge, [`ApiConnectionMode::Direct`] is returned.
pub struct ApiConnectionModeProvider {
    cache_dir: PathBuf,
    /// Used for selecting a Relay when the `Bridges` access method is used.
    relay_selector: RelaySelector,
    retry_attempt: u32,
    current_task: Option<Pin<Box<dyn Future<Output = ApiConnectionMode> + Send>>>,
}

impl Stream for ApiConnectionModeProvider {
    type Item = ApiConnectionMode;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Poll the current task
        if let Some(task) = self.current_task.as_mut() {
            return match task.as_mut().poll(cx) {
                Poll::Ready(mode) => {
                    self.current_task = None;
                    Poll::Ready(Some(mode))
                }
                Poll::Pending => Poll::Pending,
            };
        }

        let connection_mode = self.new_task(self.retry_attempt);
        self.retry_attempt = self.retry_attempt.wrapping_add(1);

        let cache_dir = self.cache_dir.clone();
        self.current_task = Some(Box::pin(async move {
            if let Err(error) = connection_mode.save(&cache_dir).await {
                log::debug!(
                    "{}",
                    error.display_chain_with_msg("Failed to save API endpoint")
                );
            }
            connection_mode
        }));

        self.poll_next(cx)
    }
}

impl ApiConnectionModeProvider {
    pub(crate) fn new(cache_dir: PathBuf, relay_selector: RelaySelector) -> Self {
        Self {
            cache_dir,

            relay_selector,
            retry_attempt: 0,

            current_task: None,
        }
    }

    fn should_use_bridge(retry_attempt: u32) -> bool {
        retry_attempt % 3 > 0
    }

    fn should_use_direct(retry_attempt: u32) -> bool {
        // TODO: Change back before comitting!
        false
        // !Self::should_use_bridge(retry_attempt)
    }

    /// Return a new connection mode to be used for the API connection.
    ///
    /// TODO: Figure out an appropriate algorithm for selecting between the available AccessMethods.
    /// We need to be able to poll the daemon's settings to find out which access methods are available & active.
    /// For now, we a
    fn new_task(&self, retry_attempt: u32) -> ApiConnectionMode {
        if Self::should_use_direct(retry_attempt) {
            ApiConnectionMode::Direct
        } else {
            self.relay_selector
                .get_bridge_forced()
                .and_then(|settings| match settings {
                    ProxySettings::Shadowsocks(ss_settings) => {
                        let ss_settings: api_access_method::Shadowsocks =
                            api_access_method::Shadowsocks::new(
                                ss_settings.peer,
                                ss_settings.cipher,
                                ss_settings.password,
                            );
                        println!("Using bridge mode to access the API! {:?}", ss_settings);
                        Some(ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(
                            ss_settings,
                        )))
                    }
                    _ => {
                        log::error!("Received unexpected proxy settings type");
                        None
                    }
                })
                .unwrap_or(ApiConnectionMode::Direct)
        }
    }
}

/// Notifies the tunnel state machine that the API (real or proxied) endpoint has
/// changed. [ApiEndpointUpdaterHandle::callback()] creates a callback that may
/// be passed to the `mullvad-api` runtime.
pub(super) struct ApiEndpointUpdaterHandle {
    tunnel_cmd_tx: Arc<Mutex<Option<Weak<mpsc::UnboundedSender<TunnelCommand>>>>>,
}

impl ApiEndpointUpdaterHandle {
    pub fn new() -> Self {
        Self {
            tunnel_cmd_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_tunnel_command_tx(&self, tunnel_cmd_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>) {
        *self.tunnel_cmd_tx.lock().unwrap() = Some(tunnel_cmd_tx);
    }

    pub fn callback(&self) -> impl ApiEndpointUpdateCallback {
        let tunnel_tx = self.tunnel_cmd_tx.clone();
        move |address: SocketAddr| {
            let inner_tx = tunnel_tx.clone();
            async move {
                let tunnel_tx = if let Some(Some(tunnel_tx)) = { inner_tx.lock().unwrap().as_ref() }
                    .map(|tx: &Weak<mpsc::UnboundedSender<TunnelCommand>>| tx.upgrade())
                {
                    tunnel_tx
                } else {
                    log::error!("Rejecting allowed endpoint: Tunnel state machine is not running");
                    return false;
                };
                let (result_tx, result_rx) = oneshot::channel();
                let _ = tunnel_tx.unbounded_send(TunnelCommand::AllowEndpoint(
                    get_allowed_endpoint(address),
                    result_tx,
                ));
                // Wait for the firewall policy to be updated.
                let _ = result_rx.await;
                log::debug!("API endpoint: {}", address);
                true
            }
        }
    }
}

pub(super) fn get_allowed_endpoint(api_address: SocketAddr) -> AllowedEndpoint {
    let endpoint = Endpoint::from_socket_address(api_address, TransportProtocol::Tcp);

    #[cfg(windows)]
    let daemon_exe = std::env::current_exe().expect("failed to obtain executable path");
    #[cfg(windows)]
    let clients = vec![
        daemon_exe
            .parent()
            .expect("missing executable parent directory")
            .join("mullvad-problem-report.exe"),
        daemon_exe,
    ];

    AllowedEndpoint {
        #[cfg(windows)]
        clients,
        endpoint,
    }
}

pub(crate) fn forward_offline_state(
    api_availability: ApiAvailabilityHandle,
    mut offline_state_rx: mpsc::UnboundedReceiver<bool>,
) {
    tokio::spawn(async move {
        let initial_state = offline_state_rx
            .next()
            .await
            .expect("missing initial offline state");
        api_availability.set_offline(initial_state);
        while let Some(is_offline) = offline_state_rx.next().await {
            api_availability.set_offline(is_offline);
        }
    });
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
