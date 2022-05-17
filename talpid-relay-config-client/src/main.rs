use std::net::{IpAddr, Ipv4Addr};

use talpid_types::net::wireguard::PrivateKey;

#[tokio::main]
async fn main() {
    let current_private_key = PrivateKey::new_from_random();

    let (private_key, psk) = talpid_relay_config_client::push_pq_key(
        IpAddr::V4(Ipv4Addr::new(10, 64, 0, 1)),
        current_private_key.public_key(),
    )
    .await
    .unwrap();

    println!("private key: {:?}", private_key);
    println!("psk: {:?}", psk);
}
