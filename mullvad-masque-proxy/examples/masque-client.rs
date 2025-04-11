use anyhow::Context;
use clap::Parser;
use mullvad_masque_proxy::client::Error;
use tokio::net::UdpSocket;

use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
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

    /// Bind address
    #[arg(long, short = 'b', default_value = "127.0.0.1:0")]
    bind_addr: SocketAddr,

    /// Maximum packet size
    #[arg(long, short = 'S', default_value = "1280")]
    mtu: u16,

    /// Maximum duration of inactivity (in seconds) until the tunnel times out.
    /// Inactivity happens when no data is sent over the proxy.
    #[arg(long, short = 'i', value_parser = duration_from_seconds)]
    idle_timeout: Option<Duration>,
}

/// Parse a duration from a decimal number of seconds
fn duration_from_seconds(s: &str) -> anyhow::Result<Duration> {
    let seconds: f64 = s.parse().context("Expected a decimal number, e.g. 1.0")?;
    Ok(Duration::from_secs_f64(seconds))
}

#[tokio::main]
async fn main() {
    let ClientArgs {
        server_addr,
        target_addr,
        root_cert_path,
        server_hostname,
        bind_addr,
        mtu,
        idle_timeout,
    } = ClientArgs::parse();

    let tls_config = match root_cert_path {
        Some(path) => mullvad_masque_proxy::client::client_tls_config_from_cert_path(path.as_ref())
            .expect("Failed to get TLS config"),
        None => mullvad_masque_proxy::client::default_tls_config(),
    };

    let _keylog = rustls::KeyLogFile::new();

    let local_socket = UdpSocket::bind(bind_addr)
        .await
        .expect("Failed to bind address");
    let local_addr = local_socket.local_addr().unwrap();
    println!("Listening on {local_addr}");

    let client = mullvad_masque_proxy::client::Client::connect_with_tls_config(
        local_socket,
        server_addr,
        (Ipv4Addr::UNSPECIFIED, 0).into(),
        target_addr,
        &server_hostname,
        tls_config,
        mtu,
        idle_timeout,
    )
    .await;
    if let Err(err) = &client {
        println!("ERROR: {:?}", err);
        if let Error::Connection(err) = err {
            println!("ERROR: {}", err);
        }
    }
    client
        .expect("Failed to connect client")
        .run()
        .await
        .unwrap();
}
