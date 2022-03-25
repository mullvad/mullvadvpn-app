use crate::DaemonEventSender;
use futures::{
    channel::{mpsc, oneshot},
    stream, Stream, StreamExt,
};
use mullvad_api::{proxy::ApiConnectionMode, ApiEndpointUpdateCallback};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, Weak},
};
use talpid_core::{mpsc::Sender, tunnel_state_machine::TunnelCommand};
use talpid_types::{
    net::{AllowedEndpoint, Endpoint, TransportProtocol},
    ErrorExt,
};

pub(crate) struct ApiConnectionModeRequest {
    pub response_tx: oneshot::Sender<ApiConnectionMode>,
    pub retry_attempt: u32,
}

/// Returns a stream that returns the next API bridge to try.
/// `initial_config` refers to the first config returned by the stream. The daemon is not notified
/// of this.
pub(crate) fn create_api_config_provider(
    daemon_sender: DaemonEventSender<ApiConnectionModeRequest>,
    initial_config: ApiConnectionMode,
) -> impl Stream<Item = ApiConnectionMode> + Unpin {
    struct Context {
        attempt: u32,
        daemon_sender: DaemonEventSender<ApiConnectionModeRequest>,
    }

    let ctx = Context {
        attempt: 1,
        daemon_sender,
    };

    Box::pin(
        stream::once(async move { initial_config }).chain(stream::unfold(
            ctx,
            |mut ctx| async move {
                ctx.attempt = ctx.attempt.wrapping_add(1);
                let (response_tx, response_rx) = oneshot::channel();

                let _ = ctx.daemon_sender.send(ApiConnectionModeRequest {
                    response_tx,
                    retry_attempt: ctx.attempt,
                });

                let new_config = response_rx.await.unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to receive API proxy config")
                    );
                    // Fall back on unbridged connection
                    ApiConnectionMode::Direct
                });

                Some((new_config, ctx))
            },
        )),
    )
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
