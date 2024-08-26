use super::Obfuscator;
use async_trait::async_trait;
use shadowsocks::{
    config::{ServerConfig, ServerType},
    context::Context,
    crypto::CipherKind,
    net::ConnectOpts,
    relay::{udprelay::proxy_socket::ProxySocketError, Address},
    ProxySocket,
};
use std::{io, net::SocketAddr, sync::Arc};
use tokio::{net::UdpSocket, sync::oneshot};

const SHADOWSOCKS_CIPHER: CipherKind = CipherKind::AES_256_GCM;
const SHADOWSOCKS_PASSWORD: &str = "mullvad";

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to bind local UDP socket
    #[error("Failed to bind UDP socket")]
    BindUdp(#[source] io::Error),
    /// Missing UDP listener address
    #[error("Failed to retrieve UDP socket bind address")]
    GetUdpLocalAddress(#[source] io::Error),
    /// Failed to wait for UDP client
    #[error("Failed to wait for UDP client")]
    WaitForUdpClient(#[source] io::Error),
    /// Failed to create UDP stream
    #[error("Failed to create UDP stream")]
    CreateUdpStream(#[source] io::Error),
    /// Failed to connect to Shadowsocks endpoint
    #[error("Failed to connect to Shadowsocks endpoint")]
    ConnectShadowsocks(#[from] ProxySocketError),
    /// Failed to receive remote socket descriptor
    #[error("Failed to receive remote socket descriptor")]
    ReceiveRemoteFd,
}

pub struct Shadowsocks {
    udp_client_addr: SocketAddr,
    server: tokio::task::JoinHandle<Result<()>>,
    // The receiver will implicitly shut down when this is dropped
    _shutdown_tx: oneshot::Sender<()>,
    #[cfg(target_os = "android")]
    outbound_fd: i32,
}

#[derive(Debug)]
pub struct Settings {
    /// Remote Shadowsocks endpoint
    pub shadowsocks_endpoint: SocketAddr,
    /// Remote WireGuard endpoint
    pub wireguard_endpoint: SocketAddr,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

impl Shadowsocks {
    pub(crate) async fn new(settings: &Settings) -> Result<Self> {
        let (local_udp_socket, udp_client_addr) =
            create_local_udp_socket(settings.shadowsocks_endpoint.is_ipv4()).await?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        #[cfg(target_os = "android")]
        let (outbound_fd_tx, outbound_fd_rx) = oneshot::channel();

        let server = tokio::spawn(run_obfuscation(
            local_udp_socket,
            settings.shadowsocks_endpoint,
            settings.wireguard_endpoint,
            shutdown_rx,
            #[cfg(target_os = "android")]
            outbound_fd_tx,
            #[cfg(target_os = "linux")]
            settings.fwmark,
        ));

        #[cfg(target_os = "android")]
        let outbound_fd = outbound_fd_rx.await.map_err(|_| Error::ReceiveRemoteFd)?;

        Ok(Shadowsocks {
            udp_client_addr,
            server,
            _shutdown_tx: shutdown_tx,
            #[cfg(target_os = "android")]
            outbound_fd,
        })
    }
}

async fn run_obfuscation(
    local_udp_socket: UdpSocket,
    shadowsocks_endpoint: SocketAddr,
    wireguard_endpoint: SocketAddr,
    shutdown_rx: oneshot::Receiver<()>,
    #[cfg(target_os = "android")] outbound_fd_tx: oneshot::Sender<i32>,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Result<()> {
    let shadowsocks = create_shadowsocks_client(
        shadowsocks_endpoint,
        #[cfg(target_os = "linux")]
        fwmark,
    )
    .await?;

    #[cfg(target_os = "android")]
    let _ = outbound_fd_tx.send(shadowsocks.get_outbound_raw_fd());

    wait_for_local_udp_client(&local_udp_socket)
        .await
        .map_err(Error::WaitForUdpClient)?;

    let local_udp = Arc::new(local_udp_socket);
    let shadowsocks = Arc::new(shadowsocks);

    let wg_addr = Address::SocketAddress(wireguard_endpoint);

    let mut client = tokio::spawn(handle_outgoing(
        shadowsocks.clone(),
        local_udp.clone(),
        wg_addr.clone(),
    ));
    let mut server = tokio::spawn(handle_incoming(shadowsocks, local_udp, wg_addr));

    tokio::select! {
        _ = shutdown_rx => {
            log::trace!("Stopping shadowsocks obfuscation");
        }
        _result = &mut server => log::trace!("Shadowsocks client closed"),
        _result = &mut client => log::trace!("Local UDP client closed"),
    }

    client.abort();
    server.abort();

    Ok(())
}

async fn create_shadowsocks_client(
    shadowsocks_endpoint: SocketAddr,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> std::result::Result<ProxySocket, ProxySocketError> {
    let ss_context = Context::new_shared(ServerType::Local);
    let ss_config: ServerConfig = ServerConfig::new(
        shadowsocks_endpoint,
        SHADOWSOCKS_PASSWORD,
        SHADOWSOCKS_CIPHER,
    );
    let connect_opts = ConnectOpts {
        #[cfg(target_os = "linux")]
        fwmark,
        ..Default::default()
    };
    ProxySocket::connect_with_opts(ss_context, &ss_config, &connect_opts).await
}

async fn create_local_udp_socket(ipv4: bool) -> Result<(UdpSocket, SocketAddr)> {
    let random_bind_addr = if ipv4 {
        SocketAddr::new("127.0.0.1".parse().unwrap(), 0)
    } else {
        SocketAddr::new("::1".parse().unwrap(), 0)
    };
    let local_udp_socket = UdpSocket::bind(random_bind_addr)
        .await
        .map_err(Error::BindUdp)?;
    let udp_client_addr = local_udp_socket
        .local_addr()
        .map_err(Error::GetUdpLocalAddress)?;

    Ok((local_udp_socket, udp_client_addr))
}

/// Wait for a client to connect to `udp_listener` and connect the socket to that address
async fn wait_for_local_udp_client(udp_listener: &UdpSocket) -> io::Result<()> {
    log::trace!("Waiting for UDP socket client");
    let client_addr = udp_listener.peek_sender().await?;

    log::trace!("UDP connection from {client_addr}");
    udp_listener.connect(client_addr).await
}

async fn handle_outgoing(
    ss_write: Arc<ProxySocket>,
    local_udp_read: Arc<UdpSocket>,
    wg_addr: Address,
) {
    let mut rx_buffer = vec![0u8; u16::MAX as usize];

    loop {
        let read_n = match local_udp_read.recv(&mut rx_buffer).await {
            Ok(read_n) => read_n,
            Err(error) => {
                log::error!("Failed to read from local UDP socket: {error}");
                break;
            }
        };

        if let Err(error) = ss_write.send(&wg_addr, &rx_buffer[0..read_n]).await {
            if is_fatal_socket_error(&error) {
                log::error!("Failed to write to Shadowsocks client: {error}");
                break;
            }
            log::trace!("Failed to write to Shadowsocks client: {error}");
        }
    }
}

async fn handle_incoming(
    ss_read: Arc<ProxySocket>,
    local_udp_write: Arc<UdpSocket>,
    wg_addr: Address,
) {
    let mut rx_buffer = vec![0u8; u16::MAX as usize];

    loop {
        let (read_n, addr, _ctrl) = match ss_read.recv(&mut rx_buffer).await {
            Ok(read_n) => read_n,
            Err(error) => {
                log::error!("Failed to read from Shadowsocks client: {error}");
                break;
            }
        };

        if addr != wg_addr {
            log::trace!("Ignoring packet from unexpected source: {addr}");
            continue;
        }

        if let Err(error) = local_udp_write.send(&rx_buffer[0..read_n]).await {
            log::error!("Failed to write to local UDP socket: {error}");
            if is_fatal_socket_io_error(&error) {
                break;
            }
        }
    }
}

#[async_trait]
impl Obfuscator for Shadowsocks {
    fn endpoint(&self) -> SocketAddr {
        self.udp_client_addr
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        match self.server.await {
            Ok(result) => result.map_err(crate::Error::RunShadowsocksObfuscator),
            Err(_err) if _err.is_cancelled() => Ok(()),
            Err(_err) => panic!("server handle panicked"),
        }
    }

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        self.outbound_fd
    }
}

/// Return whether retrying is a lost cause
fn is_fatal_socket_error(error: &ProxySocketError) -> bool {
    matches!(error, ProxySocketError::IoError(e) if is_fatal_socket_io_error(e))
}

fn is_fatal_socket_io_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotConnected
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
    )
}
