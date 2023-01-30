//! A server implementation of the tuncfg PskExchangeV1 RPC to test
//! the client side implementation.

#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("tunnel_config");
}
use classic_mceliece_rust::{PublicKey, CRYPTO_PUBLICKEYBYTES};
use proto::{
    post_quantum_secure_server::{PostQuantumSecure, PostQuantumSecureServer},
    PskRequestV1, PskResponseV1,
};
use talpid_types::net::wireguard::PresharedKey;

use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct PostQuantumSecureImpl {}

#[tonic::async_trait]
impl PostQuantumSecure for PostQuantumSecureImpl {
    async fn psk_exchange_v1(
        &self,
        request: Request<PskRequestV1>,
    ) -> Result<Response<PskResponseV1>, Status> {
        let mut rng = rand::thread_rng();
        let request = request.into_inner();

        println!("wg_pubkey: {:?}", request.wg_pubkey);
        println!("wg_psk_pubkey: {:?}", request.wg_psk_pubkey);

        // The ciphertexts that will be returned to the client
        let mut ciphertexts = Vec::new();
        // The final PSK that is computed by XORing together all the KEM outputs.
        let mut psk_data = Box::new([0u8; 32]);

        for kem_pubkey in request.kem_pubkeys {
            println!("\tKEM algorithm: {}", kem_pubkey.algorithm_name);
            let (ciphertext, shared_secret) = match kem_pubkey.algorithm_name.as_str() {
                "Classic-McEliece-460896f-round3" => {
                    let key_data: [u8; CRYPTO_PUBLICKEYBYTES] =
                        kem_pubkey.key_data.as_slice().try_into().unwrap();
                    let public_key = PublicKey::from(&key_data);
                    let (ciphertext, shared_secret) =
                        classic_mceliece_rust::encapsulate_boxed(&public_key, &mut rng);
                    (ciphertext.as_array().to_vec(), *shared_secret.as_array())
                }
                "Kyber1024" => {
                    let public_key = kem_pubkey.key_data.as_slice();
                    let (ciphertext, shared_secret) =
                        pqc_kyber::encapsulate(public_key, &mut rng).unwrap();
                    (ciphertext.to_vec(), shared_secret)
                }
                name => panic!("Unsupported KEM algorithm: {name}"),
            };

            ciphertexts.push(ciphertext);
            println!("\tshared secret: {shared_secret:?}");
            for (psk_byte, shared_secret_byte) in psk_data.iter_mut().zip(shared_secret.iter()) {
                *psk_byte ^= shared_secret_byte;
            }
        }

        let psk = PresharedKey::from(psk_data);
        println!("psk: {psk:?}");
        println!("==============================================");
        Ok(Response::new(PskResponseV1 { ciphertexts }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:1337".parse()?;
    let server = PostQuantumSecureImpl::default();

    Server::builder()
        .add_service(PostQuantumSecureServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
