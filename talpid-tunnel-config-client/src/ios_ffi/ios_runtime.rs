use super::ios_tcp_connection::*;
use super::PostQuantumCancelToken;
use crate::request_ephemeral_peer;
use crate::Error;
use crate::RelayConfigService;
use libc::c_void;
use std::sync::Arc;
use std::{io, ptr};
use talpid_types::net::wireguard::{PrivateKey, PublicKey};
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tonic::transport::channel::Endpoint;
use tower::util::service_fn;

/// # Safety
/// packet_tunnel and tcp_connection must be valid pointers to a packet tunnel and a TCP connection instances.
///
pub unsafe fn run_ios_runtime(
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel: *const c_void,
    tcp_connection: *const c_void,
) -> Result<PostQuantumCancelToken, i32> {
    match unsafe { IOSRuntime::new(pub_key, ephemeral_key, packet_tunnel, tcp_connection) } {
        Ok(runtime) => {
            let token = runtime.cancel_token_tx.clone();

            runtime.run();
            Ok(PostQuantumCancelToken {
                context: Arc::into_raw(token) as *mut _,
            })
        }
        Err(err) => {
            log::error!("Failed to create runtime {}", err);
            Err(-1)
        }
    }
}

#[derive(Clone)]
pub struct SwiftContext {
    pub packet_tunnel: *const c_void,
    pub tcp_connection: *const c_void,
}

unsafe impl Send for SwiftContext {}
unsafe impl Sync for SwiftContext {}

struct IOSRuntime {
    runtime: tokio::runtime::Runtime,
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel: SwiftContext,
    cancel_token_tx: Arc<mpsc::UnboundedSender<()>>,
    cancel_token_rx: mpsc::UnboundedReceiver<()>,
}

impl IOSRuntime {
    pub unsafe fn new(
        pub_key: [u8; 32],
        ephemeral_key: [u8; 32],
        packet_tunnel: *const libc::c_void,
        tcp_connection: *const c_void,
    ) -> io::Result<Self> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()?;

        let context = SwiftContext {
            packet_tunnel,
            tcp_connection,
        };

        let (tx, rx) = mpsc::unbounded_channel();

        Ok(Self {
            runtime,
            pub_key,
            ephemeral_key,
            packet_tunnel: context,
            cancel_token_tx: Arc::new(tx),
            cancel_token_rx: rx,
        })
    }

    pub fn run(self) {
        std::thread::spawn(move || {
            self.run_service_inner();
        });
    }

    pub async fn ios_tcp_client(ctx: SwiftContext) -> Result<RelayConfigService, Error> {
        let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
        let conn = endpoint
            .connect_with_connector(service_fn(move |_| {
                let ctx = ctx.clone();
                let tcp_provider = unsafe { IosTcpProvider::new(ctx.tcp_connection) };
                async move { Ok::<_, Error>(tcp_provider) }
            }))
            .await
            .map_err(Error::GrpcConnectError)?;

        Ok(RelayConfigService::new(conn))
    }

    fn run_service_inner(self) {
        let Self {
            runtime,
            mut cancel_token_rx,
            ..
        } = self;

        let packet_tunnel_ptr = self.packet_tunnel.packet_tunnel;
        runtime.block_on(async move {
            let async_provider = match Self::ios_tcp_client(self.packet_tunnel).await {
                Ok(async_provider) => async_provider,
                Err(error) => {
                    log::error!("Failed to create iOS TCP client: {error}");
                    unsafe {
                        swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null(), ptr::null());
                    }
                    return;
                }
            };
            let ephemeral_pub_key = PrivateKey::from(self.ephemeral_key).public_key();

            tokio::select! {
                ephemeral_peer = request_ephemeral_peer(
                    PublicKey::from(self.pub_key),
                    ephemeral_pub_key,
                    true,
                    false,
                    async_provider,
                ) =>  {
                    match ephemeral_peer {
                        Ok(peer) => {
                            match peer.psk {
                                Some(preshared_key) => unsafe {
                                    let preshared_key_bytes = preshared_key.as_bytes();
                                    swift_post_quantum_key_ready(packet_tunnel_ptr, preshared_key_bytes.as_ptr(), self.ephemeral_key.as_ptr());
                                },
                                None => unsafe {
                                    swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null(), ptr::null());
                                }
                            }
                        },
                        Err(_) => unsafe {
                            swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null(), ptr::null());
                        }
                    }
                    // match key_result {
                    //     Ok(preshared_key) => unsafe {
                    //         let preshared_key_bytes = preshared_key.as_bytes();
                    //         swift_post_quantum_key_ready(packet_tunnel_ptr, preshared_key_bytes.as_ptr(), self.ephemeral_key.as_ptr());
                    //     },
                    //     Err(_) => unsafe {
                    //         swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null(), ptr::null());
                    //     },
                    // }
                }

                _ = cancel_token_rx.recv() => {
                    // The swift runtime pre emptively cancelled the key exchange, nothing to do here.
                }
            }
        });
    }
}
