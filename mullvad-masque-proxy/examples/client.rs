use mullvad_masque_proxy::client::Error;
use tokio::net::UdpSocket;

use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let server_addr = args
        .next()
        .unwrap()
        .parse::<SocketAddr>()
        .expect("Failed to parse socket addr");

    let server_host = args.next().unwrap().to_string();

    let target_addr = args
        .next()
        .unwrap()
        .parse::<SocketAddr>()
        .expect("Failed to parse socket addr");

    let bind_port = args
        .next()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(0);

    let cert_path = args.next();
    let tls_config = match cert_path {
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
    println!("Listening on {local_addr}");

    let client = mullvad_masque_proxy::client::Client::connect_with_tls_config(
        local_socket,
        server_addr,
        (Ipv4Addr::UNSPECIFIED, 0).into(),
        target_addr,
        &server_host,
        tls_config,
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
