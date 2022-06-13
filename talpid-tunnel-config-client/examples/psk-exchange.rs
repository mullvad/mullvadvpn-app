use std::{
    io,
    net::{IpAddr, Ipv4Addr},
};

use talpid_types::net::wireguard::PublicKey;

#[tokio::main]
async fn main() {
    println!("Make sure you're connected to a WireGuard peer and enter your public key: ");

    let mut pubkey_s = String::new();
    io::stdin()
        .read_line(&mut pubkey_s)
        .expect("Failed to read from stdin");
    let pubkey = PublicKey::from_base64(pubkey_s.trim()).expect("Invalid public key");

    let (private_key, psk) =
        talpid_tunnel_config_client::push_pq_key(IpAddr::V4(Ipv4Addr::new(10, 64, 0, 1)), pubkey)
            .await
            .unwrap();

    println!("private key: {:?}", private_key);
    println!("psk: {:?}", psk);
}
