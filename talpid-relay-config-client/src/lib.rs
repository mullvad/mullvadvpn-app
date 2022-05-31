pub mod client;
mod kem;
pub use client::*;

#[derive(Debug)]
pub enum Error {
    GrpcTransportError(tonic::transport::Error),
    GrpcError(tonic::Status),
    KeyGenerationFailed,
    DecapsulationError,
    InvalidCiphertext,
}
