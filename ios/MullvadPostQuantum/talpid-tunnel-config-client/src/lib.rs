use std::ptr;

use libc::c_void;

use std::io;

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

pub unsafe fn run_ios_runtime(
    pub_key: [u8; 32],
    ephemeral_pub_key: [u8; 32],
    packet_tunnel: *const c_void,
    tcp_connection: *const c_void,
) -> i32 {
    match IOSRuntime::new(pub_key, ephemeral_pub_key, packet_tunnel, tcp_connection) {
        Ok(runtime) => {
            runtime.run();
            0
        }
        Err(err) => {
            log::error!("Failed to create runtime {}", err);
            err.raw_os_error().unwrap_or(-1)
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
    ephemeral_public_key: [u8; 32],
    packet_tunnel: SwiftContext,
}

impl IOSRuntime {
    pub unsafe fn new(
        pub_key: [u8; 32],
        ephemeral_public_key: [u8; 32],
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

        Ok(Self {
            runtime,
            pub_key,
            ephemeral_public_key,
            packet_tunnel: context,
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
        let Self { runtime, .. } = self;

        let packet_tunnel_ptr = self.packet_tunnel.packet_tunnel;
        runtime.block_on(async move {
            let mut async_provider = match Self::ios_tcp_client(self.packet_tunnel).await {
                Ok(async_provider) => async_provider,
                Err(error) => {
                    log::error!("Failed to create iOS TCP client: {error}");
                    unsafe {
                        swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null_mut());
                    }
                    return;
                }
            };
            let preshared_key = talpid_tunnel_config_client::push_pq_inner(
                &mut async_provider,
                PublicKey::from(self.pub_key),
                PublicKey::from(self.ephemeral_public_key),
            )
            .await;

            match preshared_key {
                Ok(key) => unsafe {
                    let bytes = key.as_bytes();
                    swift_post_quantum_key_ready(packet_tunnel_ptr, bytes.as_ptr());
                },
                Err(_) => unsafe {
                    swift_post_quantum_key_ready(packet_tunnel_ptr, ptr::null_mut());
                },
            }
        });
    }
}
