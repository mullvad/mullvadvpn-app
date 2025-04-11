use clap::Parser;
use mullvad_masque_proxy::client::Error;
use tokio::net::UdpSocket;

use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[derive(Parser, Debug)]
pub struct ClientArgs {
    /// Destination to forward to
    #[arg(long, short = 't')]
    target_addr: SocketAddr,

    /// Path to cert
    #[arg(long, short = 'c', required = false)]
    root_cert_path: Option<PathBuf>,

    /// Server address
    #[arg(long, short = 's')]
    server_addr: SocketAddr,

    /// Server hostname/authority
    #[arg(long, short = 'H')]
    server_hostname: String,

    /// Local bind port
    #[arg(long, short = 'p', default_value = "0")]
    bind_port: u16,

    /// Maximum packet size
    #[arg(long, short = 'S', default_value = "1280")]
    mtu: u16,

    /// fwmark to use for the `server_addr` connection
    #[cfg(target_os = "linux")]
    #[arg(long)]
    fwmark: Option<u16>,
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let ClientArgs {
        server_addr,
        target_addr,
        root_cert_path,
        server_hostname,
        bind_port,
        mtu,
        #[cfg(target_os = "linux")]
        fwmark,
    } = ClientArgs::parse();

    let tls_config = match root_cert_path {
        Some(path) => mullvad_masque_proxy::client::client_tls_config_from_cert_path(path.as_ref())
            .expect("Failed to get TLS config"),
        None => mullvad_masque_proxy::client::default_tls_config(),
    };

    let _keylog = rustls::KeyLogFile::new();

    let unbound_local_addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, bind_port).into();
    let local_socket = UdpSocket::bind(unbound_local_addr)
        .await
        .expect("Failed to bind address");
    let local_addr = local_socket.local_addr().unwrap();

    log::info!("Listening on {local_addr}");

    let client = mullvad_masque_proxy::client::Client::connect_with_tls_config(
        local_socket,
        server_addr,
        (Ipv4Addr::UNSPECIFIED, 0).into(),
        target_addr,
        &server_hostname,
        tls_config,
        mtu,
        #[cfg(target_os = "linux")]
        fwmark,
    )
    .await;
    if let Err(err) = &client {
        log::error!("ERROR: {:?}", err);
        if let Error::Connection(err) = err {
            log::error!("ERROR: {}", err);
        }
    }
    client
        .expect("Failed to connect client")
        .run()
        .await
        .unwrap();
}
