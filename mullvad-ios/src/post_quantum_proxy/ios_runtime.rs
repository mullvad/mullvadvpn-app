use super::{ios_tcp_connection::*, EphemeralPeerCancelToken};
use libc::c_void;
use std::{
    future::Future,
    io,
    pin::Pin,
    ptr,
    sync::{Arc, Mutex},
};
use talpid_tunnel_config_client::{request_ephemeral_peer_with, Error, RelayConfigService};
use talpid_types::net::wireguard::{PrivateKey, PublicKey};
use tokio::runtime::Handle as TokioHandle;
use tonic::transport::channel::Endpoint;
use tower::util::service_fn;

/// # Safety
/// packet_tunnel and tcp_connection must be valid pointers to a packet tunnel and a TCP connection
/// instances.
pub unsafe fn run_post_quantum_psk_exchange(
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel: *const c_void,
    tcp_connection: *const c_void,
    peer_exchange_timeout: u64,
    tokio_handle: TokioHandle,
    enable_post_quantum: bool,
    enable_daita: bool,
) -> Result<EphemeralPeerCancelToken, Error> {
    match unsafe {
        IOSRuntime::new(
            pub_key,
            ephemeral_key,
            packet_tunnel,
            tcp_connection,
            peer_exchange_timeout,
            enable_post_quantum,
            enable_daita,
        )
    } {
        Ok(runtime) => {
            let token = runtime.packet_tunnel.tcp_connection.clone();

            runtime.run(tokio_handle);
            Ok(EphemeralPeerCancelToken {
                context: Arc::into_raw(token) as *mut _,
            })
        }
        Err(err) => {
            log::error!("Failed to create runtime {}", err);
            Err(Error::UnableToCreateRuntime)
        }
    }
}

#[derive(Clone)]
pub struct SwiftContext {
    pub packet_tunnel: *const c_void,
    pub tcp_connection: Arc<Mutex<ConnectionContext>>,
}

unsafe impl Send for SwiftContext {}
unsafe impl Sync for SwiftContext {}

struct IOSRuntime {
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel: SwiftContext,
    peer_exchange_timeout: u64,
    enable_post_quantum: bool,
    enable_daita: bool,
}

impl IOSRuntime {
    pub unsafe fn new(
        pub_key: [u8; 32],
        ephemeral_key: [u8; 32],
        packet_tunnel: *const libc::c_void,
        tcp_connection: *const c_void,
        post_quantum_key_exchange_timeout: u64,
        enable_post_quantum: bool,
        enable_daita: bool,
    ) -> io::Result<Self> {
        let context = SwiftContext {
            packet_tunnel,
            tcp_connection: Arc::new(Mutex::new(ConnectionContext::new(tcp_connection))),
        };

        Ok(Self {
            pub_key,
            ephemeral_key,
            packet_tunnel: context,
            peer_exchange_timeout: post_quantum_key_exchange_timeout,
            enable_post_quantum,
            enable_daita,
        })
    }

    pub fn run(self, handle: TokioHandle) {
        handle.spawn(async move {
            self.run_service_inner().await;
        });
    }
    /// Creates a `RelayConfigService` using the in-tunnel TCP Connection provided by the Packet
    /// Tunnel Provider
    ///
    /// ## Safety
    /// It is unsafe to call this with an already used `SwiftContext`
    async unsafe fn ios_tcp_client(
        ctx: SwiftContext,
    ) -> Result<(RelayConfigService, IosTcpShutdownHandle), Error> {
        let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");

        let (tcp_provider, conn_handle) = unsafe { IosTcpProvider::new(ctx.tcp_connection) };
        // One (1) TCP connection
        let mut one_tcp_connection = Some(tcp_provider);
        let conn = endpoint
            .connect_with_connector(service_fn(
                move |_| -> Pin<Box<dyn Future<Output = _> + Send>> {
                    if let Some(connection) = one_tcp_connection.take() {
                        return Box::pin(async move { Ok::<_, Error>(connection) });
                    }
                    Box::pin(async { Err(Error::TcpConnectionExpired) })
                },
            ))
            .await
            .map_err(Error::GrpcConnectError)?;

        Ok((RelayConfigService::new(conn), conn_handle))
    }

    async fn run_service_inner(self) {
        let (async_provider, shutdown_handle) = unsafe {
            match Self::ios_tcp_client(self.packet_tunnel.clone()).await {
                Ok(result) => result,

                Err(error) => {
                    log::error!("Failed to create iOS TCP client: {error}");
                    swift_ephemeral_peer_ready(
                        self.packet_tunnel.packet_tunnel,
                        ptr::null(),
                        ptr::null(),
                    );
                    return;
                }
            }
        };
        // Use `self.ephemeral_key` as the new private key when no PQ but yes DAITA
        let ephemeral_pub_key = PrivateKey::from(self.ephemeral_key).public_key();

        tokio::select! {
            ephemeral_peer = request_ephemeral_peer_with(
                async_provider,
                PublicKey::from(self.pub_key),
                ephemeral_pub_key,
                self.enable_post_quantum,
                self.enable_daita,
            ) =>  {
                shutdown_handle.shutdown();
                if let Ok(mut connection) = self.packet_tunnel.tcp_connection.lock() {
                    connection.shutdown();
                }
                match ephemeral_peer {
                    Ok(peer) => {
                        match peer.psk {
                            Some(preshared_key) => unsafe {
                                let preshared_key_bytes = preshared_key.as_bytes();
                                swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                                    preshared_key_bytes.as_ptr(),
                                    self.ephemeral_key.as_ptr());
                            },
                            None => {
                                // Daita peer was requested, but without enabling post quantum keys
                                unsafe {
                                    swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                                        ptr::null(),
                                        self.ephemeral_key.as_ptr());
                                }
                            }
                        }
                    },
                    Err(error) => {
                        log::error!("Key exchange failed {}", error);
                        unsafe {
                            swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                                ptr::null(),
                                ptr::null());
                        }
                    }
                }
            }

            _ = tokio::time::sleep(std::time::Duration::from_secs(self.peer_exchange_timeout)) => {
                        if let Ok(mut connection) = self.packet_tunnel.tcp_connection.lock() {
                            connection.shutdown();
                        };
                        shutdown_handle.shutdown();
                        unsafe { swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                            ptr::null(),
                            ptr::null()); }
            }
        }
    }
}
