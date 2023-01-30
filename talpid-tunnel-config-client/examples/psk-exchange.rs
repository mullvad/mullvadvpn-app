//! Example client implementing the quantum resistant tunnel PSK exchange.
//! Useful to test this crate's implementation.

use std::net::IpAddr;
use talpid_types::net::wireguard::PublicKey;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let tuncfg_server_ip: IpAddr = args
        .next()
        .expect("Give tuncfg server IP as first argument")
        .parse()
        .expect("tuncfg ip argument not a valid IP");
    let pubkey_string = args
        .next()
        .expect("Give WireGuard public key as second argument");
    let pubkey = PublicKey::from_base64(pubkey_string.trim()).expect("Invalid public key");

    let (private_key, psk) = talpid_tunnel_config_client::push_pq_key(tuncfg_server_ip, pubkey)
        .await
        .unwrap();

    println!("private key: {private_key:?}");
    println!("psk: {psk:?}");
}
