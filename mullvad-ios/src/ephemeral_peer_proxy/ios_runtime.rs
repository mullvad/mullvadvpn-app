use super::{
    ios_tcp_connection::*, EphemeralPeerCancelToken, EphemeralPeerParameters, PacketTunnelBridge,
};
use libc::c_void;
use std::{
    io, ptr,
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
pub unsafe fn run_ephemeral_peer_exchange(
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel_bridge: PacketTunnelBridge,
    peer_parameters: EphemeralPeerParameters,
    tokio_handle: TokioHandle,
) -> Result<EphemeralPeerCancelToken, Error> {
    match unsafe {
        IOSRuntime::new(
            pub_key,
            ephemeral_key,
            packet_tunnel_bridge,
            peer_parameters,
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

#[derive(Clone)]
#[repr(C)]
pub struct DaitaParameters {
    pub machines: *const u8,
    pub max_padding_frac: f64,
    pub max_blocking_frac: f64,
}

struct IOSRuntime {
    pub_key: [u8; 32],
    ephemeral_key: [u8; 32],
    packet_tunnel: SwiftContext,
    peer_parameters: EphemeralPeerParameters,
}

impl IOSRuntime {
    pub unsafe fn new(
        pub_key: [u8; 32],
        ephemeral_key: [u8; 32],
        packet_tunnel_bridge: PacketTunnelBridge,
        peer_parameters: EphemeralPeerParameters,
    ) -> io::Result<Self> {
        let context = SwiftContext {
            packet_tunnel: packet_tunnel_bridge.packet_tunnel,
            tcp_connection: Arc::new(Mutex::new(ConnectionContext::new(
                packet_tunnel_bridge.tcp_connection,
            ))),
        };

        Ok(Self {
            pub_key,
            ephemeral_key,
            packet_tunnel: context,
            peer_parameters,
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
            .connect_with_connector(service_fn(move |_| {
                let connection = one_tcp_connection
                    .take()
                    .map(hyper_util::rt::tokio::TokioIo::new)
                    .ok_or(Error::TcpConnectionExpired);
                async { connection }
            }))
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
                self.peer_parameters.enable_post_quantum,
                self.peer_parameters.enable_daita,
            ) =>  {
                shutdown_handle.shutdown();
                if let Ok(mut connection) = self.packet_tunnel.tcp_connection.lock() {
                    connection.shutdown();
                }

                match ephemeral_peer {
                    Ok(peer) => {
                        let params: Option<(Vec<u8>, DaitaParameters)> = peer.daita.as_ref().map(|daita| {
                            let machines_newline_delim = daita
                                .client_machines
                                .join("\n")
                                .into_bytes();
                            // IMPORTANT: `machines_newline_delim` must not be dropped while
                            // this is read
                            let machines = machines_newline_delim.as_ptr();
                            (machines_newline_delim, DaitaParameters {
                                machines,
                                max_padding_frac: daita.max_padding_frac,
                                max_blocking_frac: daita.max_blocking_frac,
                            })
                        });
                        let params_ptr = params.as_ref().map(|(_, params)| {
                            params as *const DaitaParameters
                        })
                        .unwrap_or(std::ptr::null());

                        match peer.psk {
                            Some(preshared_key) => unsafe {
                                let preshared_key_bytes = preshared_key.as_bytes();
                                swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                                    preshared_key_bytes.as_ptr(),
                                    self.ephemeral_key.as_ptr(),
                                    params_ptr);
                            },
                            None => {
                                // Daita peer was requested, but without enabling post quantum keys
                                unsafe {
                                    swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                                        ptr::null(),
                                        self.ephemeral_key.as_ptr(),
                                        params_ptr);
                                }
                            }
                        }
                    },
                    Err(error) => {
                        log::error!("Key exchange failed {}", error);
                        unsafe {
                            swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                                ptr::null(),
                                ptr::null(),
                                ptr::null());
                        }
                    }
                }
            }

            _ = tokio::time::sleep(std::time::Duration::from_secs(self.peer_parameters.peer_exchange_timeout)) => {
                        if let Ok(mut connection) = self.packet_tunnel.tcp_connection.lock() {
                            connection.shutdown();
                        };
                        shutdown_handle.shutdown();
                        unsafe { swift_ephemeral_peer_ready(self.packet_tunnel.packet_tunnel,
                            ptr::null(),
                            ptr::null(),
                            ptr::null(),
                        ); }
            }
        }
    }
}
