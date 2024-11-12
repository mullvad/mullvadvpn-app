//! A server implementation of the tuncfg RegisterPeerV1 RPC to test
//! the client side implementation of PQ.

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("ephemeralpeer");
}
use proto::{
    ephemeral_peer_server::{EphemeralPeer, EphemeralPeerServer},
    EphemeralPeerRequestV1, EphemeralPeerResponseV1, PostQuantumResponseV1,
};
use rand::{CryptoRng, RngCore};
use talpid_types::net::wireguard::PresharedKey;

use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct EphemeralPeerImpl {}

#[tonic::async_trait]
impl EphemeralPeer for EphemeralPeerImpl {
    async fn register_peer_v1(
        &self,
        request: Request<EphemeralPeerRequestV1>,
    ) -> Result<Response<EphemeralPeerResponseV1>, Status> {
        let mut rng = rand::thread_rng();
        let request = request.into_inner();

        println!("wg_parent_pubkey: {:?}", request.wg_parent_pubkey);
        println!(
            "wg_ephemeral_peer_pubkey: {:?}",
            request.wg_ephemeral_peer_pubkey
        );
        println!("daita (no-op): {:?}", request.daita);

        let post_quantum = if let Some(post_quantum) = request.post_quantum {
            // The ciphertexts that will be returned to the client
            let mut ciphertexts = Vec::new();

            // The final PSK that is computed by XORing together all the KEM outputs.
            let mut psk_data = Box::new([0u8; 32]);

            for kem_pubkey in post_quantum.kem_pubkeys {
                println!("\tKEM algorithm: {}", kem_pubkey.algorithm_name);
                let (ciphertext, shared_secret) = match kem_pubkey.algorithm_name.as_str() {
                    "Classic-McEliece-460896f-round3" => {
                        encapsulate_classic_mceliece(kem_pubkey.key_data.as_slice(), &mut rng)
                    }
                    "ML-KEM-1024" => encapsulate_ml_kem(kem_pubkey.key_data.as_slice(), &mut rng),
                    name => panic!("Unsupported KEM algorithm: {name}"),
                };

                ciphertexts.push(ciphertext);
                println!("\tshared secret: {shared_secret:?}");
                for (psk_byte, shared_secret_byte) in psk_data.iter_mut().zip(shared_secret.iter())
                {
                    *psk_byte ^= shared_secret_byte;
                }
            }

            let psk = PresharedKey::from(psk_data);
            println!("psk: {psk:?}");
            println!("==============================================");

            Some(PostQuantumResponseV1 { ciphertexts })
        } else {
            None
        };

        Ok(Response::new(EphemeralPeerResponseV1 {
            post_quantum,
            daita: None,
        }))
    }
}

/// Generate a random shared secret and encapsulate it with the given
/// public key/encapsulation key. Returns the ciphertext to return
/// to the owner of the public key, along with the shared secret.
fn encapsulate_classic_mceliece<R: RngCore + CryptoRng>(
    public_key: &[u8],
    rng: &mut R,
) -> (Vec<u8>, [u8; 32]) {
    use classic_mceliece_rust::{PublicKey, CRYPTO_PUBLICKEYBYTES};

    let public_key_array = <[u8; CRYPTO_PUBLICKEYBYTES]>::try_from(public_key).unwrap();
    let public_key = PublicKey::from(&public_key_array);
    let (ciphertext, shared_secret) = classic_mceliece_rust::encapsulate_boxed(&public_key, rng);
    (ciphertext.as_array().to_vec(), *shared_secret.as_array())
}

/// Generate a random shared secret and encapsulate it with the given
/// public key/encapsulation key. Returns the ciphertext to return
/// to the owner of the public key, along with the shared secret.
fn encapsulate_ml_kem<R: RngCore + CryptoRng>(
    public_key: &[u8],
    rng: &mut R,
) -> (Vec<u8>, [u8; 32]) {
    use ml_kem::{kem::Encapsulate, Encoded, EncodedSizeUser, KemCore, MlKem1024};

    type EncapsulationKey = <MlKem1024 as KemCore>::EncapsulationKey;

    let encapsulation_key_array = <Encoded<EncapsulationKey>>::try_from(public_key).unwrap();
    let encapsulation_key = EncapsulationKey::from_bytes(&encapsulation_key_array);

    let (ciphertext, shared_secret) = encapsulation_key.encapsulate(rng).unwrap();

    (ciphertext.to_vec(), shared_secret.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:1337".parse()?;
    let server = EphemeralPeerImpl::default();

    Server::builder()
        .add_service(EphemeralPeerServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
