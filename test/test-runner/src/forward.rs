use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use test_rpc::net::SockHandleId;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

static SERVERS: Lazy<Mutex<HashMap<SockHandleId, Handle>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Spawn a TCP forwarder that sends TCP via `via_addr`
pub async fn start_server(
    bind_addr: SocketAddr,
    via_addr: SocketAddr,
) -> Result<(SockHandleId, SocketAddr), test_rpc::Error> {
    let next_nonce = {
        static NONCE: AtomicUsize = AtomicUsize::new(0);
        || NONCE.fetch_add(1, Ordering::Relaxed)
    };
    let id = SockHandleId(next_nonce());

    let handle = tcp_forward(bind_addr, via_addr).await.map_err(|error| {
        log::error!("Failed to start TCP forwarder listener: {error}");
        test_rpc::Error::TcpForward
    })?;

    let bind_addr = handle.local_addr();

    let mut servers = SERVERS.lock().unwrap();
    servers.insert(id, handle);

    Ok((id, bind_addr))
}

/// Stop TCP forwarder given some ID returned by `start_server`
pub fn stop_server(id: SockHandleId) -> Result<(), test_rpc::Error> {
    let handle = {
        let mut servers = SERVERS.lock().unwrap();
        servers.remove(&id)
    };

    if let Some(handle) = handle {
        handle.close();
    }
    Ok(())
}

struct Handle {
    handle: tokio::task::JoinHandle<()>,
    bind_addr: SocketAddr,
    clients: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl Handle {
    pub fn close(&self) {
        self.handle.abort();

        let mut clients = self.clients.lock().unwrap();
        for client in clients.drain(..) {
            client.abort();
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.close();
    }
}

/// Forward TCP traffic via `proxy_addr`
async fn tcp_forward(
    bind_addr: SocketAddr,
    proxy_addr: SocketAddr,
) -> Result<Handle, test_rpc::Error> {
    let listener = TcpListener::bind(&bind_addr).await.map_err(|error| {
        log::error!("Failed to bind TCP forward socket: {error}");
        test_rpc::Error::TcpForward
    })?;
    let bind_addr = listener.local_addr().map_err(|error| {
        log::error!("Failed to get TCP socket addr: {error}");
        test_rpc::Error::TcpForward
    })?;

    let clients = Arc::new(Mutex::new(vec![]));

    let clients_copy = clients.clone();

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut client, _addr)) => {
                    let client_handle = tokio::spawn(async move {
                        let mut proxy = match TcpStream::connect(proxy_addr).await {
                            Ok(proxy) => proxy,
                            Err(error) => {
                                log::error!("failed to connect to TCP proxy: {error}");
                                return;
                            }
                        };

                        if let Err(error) =
                            tokio::io::copy_bidirectional(&mut client, &mut proxy).await
                        {
                            log::error!("copy_directional failed: {error}");
                        }
                    });
                    clients_copy.lock().unwrap().push(client_handle);
                }
                Err(error) => {
                    log::error!("failed to accept TCP client: {error}");
                }
            }
        }
    });
    Ok(Handle {
        handle,
        bind_addr,
        clients,
    })
}
