use std::env::args;

use mullvad_encrypted_dns_proxy::{config_resolver, forwarder};
use tokio::net::TcpListener;

/// This can be tested out by using curl:
/// `curl https://api.mullvad.net:$port/api/v1/relays --resolve api.mullvad.net:$port:$addr`
///  where $addr and $port are the listening address of the proxy (bind_addr).
#[tokio::main]
async fn main() {
    let bind_addr = args().nth(1).unwrap_or("127.0.0.1:0".to_string());

    let configs =
        config_resolver::resolve_configs(config_resolver::default_resolvers(), "frakta.eu")
            .await
            .expect("Failed to resolve configs");

    let proxy_config = configs
        .into_iter()
        .find(|c| c.obfuscation.is_some())
        .expect("No XOR config");
    println!("Proxy config in use: {:?}", proxy_config);

    let listener = TcpListener::bind(bind_addr)
        .await
        .expect("Failed to bind listener socket");

    let listen_addr = listener
        .local_addr()
        .expect("failed to obtain listen address");
    println!("Listening on {listen_addr}");

    while let Ok((client_conn, _client_addr)) = listener.accept().await {
        let connected = crate::forwarder::Forwarder::connect(&proxy_config)
            .await
            .expect("failed to connect to obfuscator");
        let _ = connected.forward(client_conn).await;
    }
}
