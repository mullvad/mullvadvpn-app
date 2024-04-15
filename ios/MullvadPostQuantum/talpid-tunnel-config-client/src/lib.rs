use std::ptr;

use libc::c_void;
use tokio::sync::mpsc;

use std::io;
use std::sync::Arc;

mod ios_ffi;
pub use ios_ffi::negotiate_post_quantum_key;

use crate::ios_tcp_connection::swift_post_quantum_key_ready;
use crate::ios_tcp_connection::IosTcpProvider;
mod ios_tcp_connection;
use talpid_tunnel_config_client::Error;
use talpid_tunnel_config_client::RelayConfigService;
use talpid_types::net::wireguard::PublicKey;
use tonic::transport::Endpoint;
use tower::service_fn;

#[repr(C)]
pub struct PostQuantumCancelToken {
    // Must keep a pointer to a valid std::sync::Arc<tokio::mpsc::UnboundedSender>
    pub context: *mut c_void,
}

impl PostQuantumCancelToken {
    /// #Safety
    /// This function can only be called when the context pointer is valid.
    unsafe fn cancel(&self) {
        // Try to take the value, if there is a value, we can safely send the message, otherwise, assume it has been dropped and nothing happens
        let send_tx: Arc<mpsc::UnboundedSender<()>> = unsafe { Arc::from_raw(self.context as _) };
        let _ = send_tx.send(());
        std::mem::forget(send_tx);
    }
}

impl Drop for PostQuantumCancelToken {
    fn drop(&mut self) {
        let _: Arc<mpsc::UnboundedSender<()>> = unsafe { Arc::from_raw(self.context as _) };
    }
}
unsafe impl Send for PostQuantumCancelToken {}

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
struct SwiftContext {
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
        let runtime = tokio::runtime::Builder::new_multi_thread()
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

    async fn ios_tcp_client(ctx: SwiftContext) -> Result<RelayConfigService, Error> {
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
            let mut async_provider = match Self::ios_tcp_client(self.packet_tunnel).await {
                Ok(async_provider) => async_provider,
                Err(error) => {
                    log::error!("Failed to create iOS TCP client: {error}");
                    unsafe {
                        swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null());
                    }
                    return;
                }
            };
            // TODO: derive the public key from the (private) ephemeral key for use here
            // let ephemeral_key = .....
            tokio::select! {
                preshared_key = talpid_tunnel_config_client::push_pq_inner(
                    &mut async_provider,
                    PublicKey::from(self.pub_key),
                    PublicKey::from(ephemeral_key),
                ) =>  {
                    match preshared_key {
                        Ok(key) => unsafe {
                            let bytes = key.as_bytes();
                            let eph_bytes = self.ephemeral_public_key.as_bytes();
                            swift_post_quantum_key_ready(packet_tunnel_ptr, bytes.as_ptr(), eph_bytes.as_ptr());
                        },
                        Err(_) => unsafe {
                            swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null(), ptr::null());
                        },
                    }
                }

                _ = cancel_token_rx.recv() => {
                    // The swift runtime pre emptively cancelled the key exchange, nothing to do here.
                }
            }
        });
    }
}
