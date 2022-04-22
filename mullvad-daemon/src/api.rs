use futures::{
    channel::{mpsc, oneshot},
    Future, Stream,
};
use mullvad_api::{
    proxy::{ApiConnectionMode, ProxyConfig},
    ApiEndpointUpdateCallback,
};
use mullvad_relay_selector::RelaySelector;
use std::{
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex, Weak},
    task::Poll,
};
use talpid_core::tunnel_state_machine::TunnelCommand;
use talpid_types::{
    net::{openvpn::ProxySettings, AllowedEndpoint, Endpoint, TransportProtocol},
    ErrorExt,
};

/// A stream that returns the next API connection mode to use for reaching the API.
///
/// When `mullvad-api` fails to contact the API, it requests a new connection mode.
/// The API can be connected to either directly (i.e., [`ApiConnectionMode::Direct`])
/// or from a bridge ([`ApiConnectionMode::Proxied`]).
///
/// * Every 3rd attempt returns [`ApiConnectionMode::Direct`].
/// * Any other attempt returns a configuration for the bridge that is closest to the selected relay
///   location and matches all bridge constraints.
/// * When no matching bridge is found, e.g. if the selected hosting providers don't match any
///   bridge, [`ApiConnectionMode::Direct`] is returned.
pub struct ApiConnectionModeProvider {
    cache_dir: PathBuf,

    selector_rx: Option<oneshot::Receiver<RelaySelector>>,

    relay_selector: Option<RelaySelector>,
    retry_attempt: u32,

    current_task: Option<Pin<Box<dyn Future<Output = ApiConnectionMode> + Send>>>,
}

impl Stream for ApiConnectionModeProvider {
    type Item = ApiConnectionMode;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // First, obtain a relay selector handle
        // Until we have obtained the handle, just return `ApiConnectionMode::Direct`.
        if let Some(rx) = self.selector_rx.as_mut() {
            match rx.try_recv() {
                Ok(Some(selector)) => {
                    self.selector_rx = None;
                    self.relay_selector = Some(selector);
                }
                Ok(None) | Err(_) => return Poll::Ready(Some(ApiConnectionMode::Direct)),
            }
        }

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

        // Create a new task.
        // Unwrapping is safe since we must have a handle at this point.
        let selector = self.relay_selector.as_ref().unwrap();

        let config = if Self::should_use_bridge(self.retry_attempt) {
            selector
                .get_bridge_forced()
                .map(|settings| match settings {
                    ProxySettings::Shadowsocks(ss_settings) => {
                        ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(ss_settings))
                    }
                    _ => {
                        log::error!("Received unexpected proxy settings type");
                        ApiConnectionMode::Direct
                    }
                })
                .unwrap_or(ApiConnectionMode::Direct)
        } else {
            ApiConnectionMode::Direct
        };

        self.retry_attempt = self.retry_attempt.wrapping_add(1);

        let cache_dir = self.cache_dir.clone();
        self.current_task = Some(Box::pin(async move {
            if let Err(error) = config.save(&cache_dir).await {
                log::debug!(
                    "{}",
                    error.display_chain_with_msg("Failed to save API endpoint")
                );
            }
            config
        }));

        return self.poll_next(cx);
    }
}

impl ApiConnectionModeProvider {
    pub(crate) fn new(cache_dir: PathBuf) -> (Self, ApiConnectionModeProviderHandle) {
        let (selector_tx, selector_rx) = oneshot::channel();

        (
            Self {
                cache_dir,

                selector_rx: Some(selector_rx),

                relay_selector: None,
                retry_attempt: 0,

                current_task: None,
            },
            ApiConnectionModeProviderHandle { tx: selector_tx },
        )
    }

    fn should_use_bridge(retry_attempt: u32) -> bool {
        retry_attempt % 3 > 0
    }
}

/// Used to initialize [ApiConnectionModeProvider]'s relay selector handle.
pub(crate) struct ApiConnectionModeProviderHandle {
    tx: oneshot::Sender<RelaySelector>,
}

impl ApiConnectionModeProviderHandle {
    pub fn set_relay_selector(self, selector: RelaySelector) {
        let _ = self.tx.send(selector);
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
                    get_allowed_endpoint(address.clone()),
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
