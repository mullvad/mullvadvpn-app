use super::{ios_tcp_connection::*, DaitaParameters, EphemeralPeerParameters, PacketTunnelBridge};
use std::{ffi::CStr, sync::Mutex, thread};
use talpid_tunnel_config_client::{
    request_ephemeral_peer_with, EphemeralPeer, Error, RelayConfigService,
};
use talpid_types::net::wireguard::{PrivateKey, PublicKey};
use tokio::{runtime::Handle as TokioHandle, task::JoinHandle};
use tonic::transport::channel::Endpoint;
use tower::util::service_fn;

const GRPC_HOST_CSTR: &CStr = c"10.64.0.1:1337";

pub struct ExchangeCancelToken {
    inner: Mutex<CancelToken>,
}

impl ExchangeCancelToken {
    fn new(tokio_handle: TokioHandle, task: JoinHandle<()>) -> Self {
        let inner = CancelToken {
            tokio_handle,
            task: Some(task),
        };
        Self {
            inner: Mutex::new(inner),
        }
    }

    /// Blocks until the associated ephemeral peer exchange task is finished.
    pub fn cancel(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(task) = inner.task.take() {
                task.abort();
                let _ = inner.tokio_handle.block_on(task);
            }
        }
    }
}

struct CancelToken {
    tokio_handle: TokioHandle,
    task: Option<JoinHandle<()>>,
}

pub struct EphemeralPeerExchange {
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel: PacketTunnelBridge,
    peer_parameters: EphemeralPeerParameters,
}

// # Safety:
// This is safe because the void pointer in PacketTunnelBridge is valid for the lifetime of the
// process where this type is intended to be used.
unsafe impl Send for EphemeralPeerExchange {}

impl EphemeralPeerExchange {
    pub fn new(
        pub_key: [u8; 32],
        ephemeral_key: [u8; 32],
        packet_tunnel: PacketTunnelBridge,
        peer_parameters: EphemeralPeerParameters,
    ) -> EphemeralPeerExchange {
        Self {
            pub_key,
            ephemeral_key,
            packet_tunnel,
            peer_parameters,
        }
    }

    pub fn run(self, tokio: TokioHandle) -> ExchangeCancelToken {
        let task = tokio.spawn(async move {
            self.run_service_inner().await;
        });

        ExchangeCancelToken::new(tokio, task)
    }

    /// Creates a `RelayConfigService` using the in-tunnel TCP Connection provided by the Packet
    /// Tunnel Provider
    async fn ios_tcp_client(
        tunnel_handle: i32,
        peer_parameters: EphemeralPeerParameters,
    ) -> Result<RelayConfigService, Error> {
        let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");

        let tcp_provider = IosTcpProvider::new(tunnel_handle, peer_parameters);

        let conn = endpoint
            // it is assumend that the service function will only be called once.
            // Yet, by its signature, it is forced to be callable multiple times.
            // The tcp_provider appeases this constraint, maybe we should rewrite this back to
            // explicitly only allow a single invocation? It is due to this mismatch between how we
            // use it and what the interface expects that we are using a oneshot channel to
            // transfer the shutdown handle.
            .connect_with_connector(service_fn(move |_| {
                let provider = tcp_provider.clone();
                async move {
                    provider
                        .connect(GRPC_HOST_CSTR)
                        .await
                        .map(hyper_util::rt::tokio::TokioIo::new)
                        .map_err(|_| Error::TcpConnectionOpen)
                }
            }))
            .await
            .map_err(Error::GrpcConnectError)?;

        Ok(RelayConfigService::new(conn))
    }

    fn report_failure(self) {
        thread::spawn(move || {
            self.packet_tunnel.fail_exchange();
        });
    }

    async fn run_service_inner(self) {
        let async_provider = match Self::ios_tcp_client(
            self.packet_tunnel.tunnel_handle,
            self.peer_parameters,
        )
        .await
        {
            Ok(result) => result,
            Err(error) => {
                log::error!("Failed to create iOS TCP client: {error}");
                self.report_failure();
                return;
            }
        };
        // Use `self.ephemeral_key` as the new private key when no PQ but yes DAITA
        let ephemeral_pub_key = PrivateKey::from(self.ephemeral_key).public_key();

        tokio::select! {
            ephemeral_peer = request_ephemeral_peer_with(
                async_provider,
                PublicKey::from(self.pub_key),
                ephemeral_pub_key,
                self.peer_parameters.enable_post_quantum,
                self.peer_parameters.enable_daita,
            ) =>  {
                match ephemeral_peer {
                    Ok(EphemeralPeer { psk, daita }) => {
                        thread::spawn(move || {
                            let Self{ ephemeral_key, packet_tunnel,  .. } = self;
                            packet_tunnel.succeed_exchange(
                                ephemeral_key,
                                psk.map(|psk| *psk.as_bytes()),
                                daita.and_then(DaitaParameters::new)
                            );
                        });
                    },
                    Err(error) => {
                        log::error!("Key exchange failed {}", error);
                        self.report_failure();
                    }
                }
            }

            _ = tokio::time::sleep(std::time::Duration::from_secs(self.peer_parameters.peer_exchange_timeout)) => {
                    self.report_failure();
            }
        }
    }
}
