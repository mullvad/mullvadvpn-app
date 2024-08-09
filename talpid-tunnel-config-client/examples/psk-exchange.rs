//! Example client implementing the quantum resistant tunnel PSK exchange.
//! Useful to test this crate's implementation.

// Usage: ./psk-exchange <tuncfg_server_ip> <wireguard_public_key>
// e. g. ./psk-exchange 10.64.0.1 NkECLsf+VbZUjve7RVN6sE3NYUcYUmUn8qpFugqbXFk=

use talpid_types::net::wireguard::{PrivateKey, PublicKey};

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let tuncfg_server_ip = args
        .next()
        .expect("Give tuncfg server IP as first argument")
        .parse()
        .expect("tuncfg IP argument not a valid IPv4");
    let public_key_string = args
        .next()
        .expect("Give WireGuard public key as second argument");
    let public_key = PublicKey::from_base64(&public_key_string).expect("Invalid public key");
    // The ephemeral peer requires an ephemeral public WireGuard key,
    // which can also be provided by other means.
    let ephemeral_private_key = PrivateKey::new_from_random();

    let ephemeral_peer = talpid_tunnel_config_client::request_ephemeral_peer(
        tuncfg_server_ip,
        public_key, // Parent connection's public key.
        ephemeral_private_key.public_key(),
        true,  // Whether to negotiate a "PQ-safe" PSK.
        false, // Whether to use DAITA (Does not work with Linux kernel WireGuard.)
    )
    .await
    .unwrap();

    println!("Private key: {ephemeral_private_key}");
    // Use fmt::Debug since Serialize is not implemented for PresharedKey.
    println!("PSK: {:?}", ephemeral_peer.psk.unwrap());
}
