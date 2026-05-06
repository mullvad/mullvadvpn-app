use crate::Error;
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SockHandleId(pub usize);

pub struct SockHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    bind_addr: SocketAddr,
}

impl SockHandle {
    pub(crate) async fn start_tcp_forward(
        client: crate::service::ServiceClient,
        bind_addr: SocketAddr,
        via_addr: SocketAddr,
    ) -> Result<Self, Error> {
        let (stop_tx, stop_rx) = oneshot::channel();

        let (id, bind_addr) = client
            .start_tcp_forward(tarpc::context::current(), bind_addr, via_addr)
            .await??;

        tokio::spawn(async move {
            let _ = stop_rx.await;

            log::trace!("Stopping TCP forward");

            if let Err(error) = client.stop_tcp_forward(tarpc::context::current(), id).await {
                log::error!("Failed to stop TCP forward: {error}");
            }
        });

        Ok(SockHandle {
            stop_tx: Some(stop_tx),
            bind_addr,
        })
    }

    pub fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

impl Drop for SockHandle {
    fn drop(&mut self) {
        self.stop()
    }
}
