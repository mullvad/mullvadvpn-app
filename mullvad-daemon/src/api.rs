use crate::DaemonEventSender;
use futures::channel::oneshot;
use mullvad_rpc::proxy::{ProxyConfig, ProxyConfigProvider};
use std::{future::Future, pin::Pin, sync::Mutex};
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;

pub(crate) struct ApiProxyRequest {
    pub response_tx: oneshot::Sender<ProxyConfig>,
    pub retry_attempt: u32,
}

pub(crate) struct MullvadProxyConfigProvider {
    retry_attempt: Mutex<u32>,
    tx: DaemonEventSender<ApiProxyRequest>,
}

impl MullvadProxyConfigProvider {
    pub fn new(tx: DaemonEventSender<ApiProxyRequest>) -> Self {
        Self {
            retry_attempt: Mutex::new(0),
            tx,
        }
    }
}

impl ProxyConfigProvider for MullvadProxyConfigProvider {
    fn next(&self) -> Pin<Box<dyn Future<Output = ProxyConfig> + Send>> {
        let (tx, rx) = oneshot::channel();

        let retry_attempt = {
            let mut attempt = self.retry_attempt.lock().unwrap();
            let prev = *attempt;
            let (next, _) = attempt.overflowing_add(1);
            *attempt = next;
            prev
        };

        let _ = self.tx.send(ApiProxyRequest {
            response_tx: tx,
            retry_attempt,
        });

        Box::pin(async move {
            rx.await.unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to receive API proxy config")
                );
                ProxyConfig::Tls
            })
        })
    }
}
