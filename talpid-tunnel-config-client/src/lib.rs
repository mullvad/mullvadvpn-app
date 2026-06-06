use gotatun::device::daita;
use proto::PostQuantumRequestV1;
use std::fmt;
#[cfg(not(target_os = "ios"))]
use std::net::SocketAddr;
#[cfg(not(target_os = "ios"))]
use std::net::{IpAddr, Ipv4Addr};
use std::time::Instant;
use talpid_types::net::wireguard::{PresharedKey, PublicKey};
use tonic::transport::Channel;
#[cfg(not(target_os = "ios"))]
use tonic::transport::Endpoint;
#[cfg(not(target_os = "ios"))]
use tower::service_fn;
use zeroize::Zeroize;

mod hqc;
mod ml_kem;
#[cfg(not(target_os = "ios"))]
mod socket;

#[cfg(not(target_os = "ios"))]
mod socket_sniffer;

#[expect(clippy::allow_attributes)]
mod proto {
    tonic::include_proto!("ephemeralpeer");
}

const DAITA_VERSION: u32 = 2;

#[derive(Debug)]
pub enum Error {
    GrpcConnectError(tonic::transport::Error),
    // TODO: Remove box when upgrading tonic to a version with
    // https://github.com/hyperium/tonic/pull/2282
    GrpcError(Box<tonic::Status>),
    MissingCiphertexts,
    InvalidCiphertextLength {
        algorithm: &'static str,
        actual: usize,
        expected: usize,
    },
    InvalidCiphertextCount {
        actual: usize,
    },
    MissingDaitaResponse,
    /// Failed to parse maybenot machines from API response.
    ParseMaybenotMachines {
        reason: String,
    },
    InvalidDaitaFraction {
        field: &'static str,
        value: f64,
    },
    #[cfg(target_os = "ios")]
    TcpConnectionOpen,
    #[cfg(target_os = "ios")]
    UnableToCreateRuntime,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GrpcConnectError(err) => write!(f, "Failed to connect to config service: {err:?}"),
            GrpcError(status) => write!(f, "RPC failed: {status}"),
            MissingCiphertexts => write!(f, "Found no ciphertexts in response"),
            InvalidCiphertextLength {
                algorithm,
                actual,
                expected,
            } => write!(
                f,
                "Expected a {expected} bytes ciphertext for {algorithm}, got {actual} bytes"
            ),
            InvalidCiphertextCount { actual } => {
                write!(f, "Expected 2 ciphertext in the response, got {actual}")
            }
            ParseMaybenotMachines { reason } => {
                write!(f, "Failed to parse Maybenot machines: {reason}")
            }
            InvalidDaitaFraction { field, value } => {
                write!(
                    f,
                    "Expected {field} to be a fraction between 0 and 1, got {value}"
                )
            }
            MissingDaitaResponse => "Expected DAITA configuration in response".fmt(f),
            #[cfg(target_os = "ios")]
            TcpConnectionOpen => "Failed to open TCP connection".fmt(f),
            #[cfg(target_os = "ios")]
            UnableToCreateRuntime => "Unable to create iOS PQ PSK runtime".fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GrpcConnectError(error) => Some(error),
            _ => None,
        }
    }
}

pub type RelayConfigService = proto::ephemeral_peer_client::EphemeralPeerClient<Channel>;

/// Port used by the tunnel config service.
pub const CONFIG_SERVICE_PORT: u16 = 1337;

pub struct EphemeralPeer {
    pub psk: Option<PresharedKey>,
    pub daita: Option<DaitaSettings>,
}

pub struct DaitaSettings {
    pub client_machines: Vec<daita::Machine>,
    pub max_decoy_frac: f64,
    pub max_delay_frac: f64,
}

/// Negotiate a short-lived peer with a PQ-safe PSK or with DAITA enabled.
#[cfg(not(target_os = "ios"))]
pub async fn request_ephemeral_peer(
    service_address: Ipv4Addr,
    parent_pubkey: PublicKey,
    ephemeral_pubkey: PublicKey,
    enable_post_quantum: bool,
    enable_daita: bool,
) -> Result<EphemeralPeer, Error> {
    log::debug!("Connecting to relay config service at {service_address}");
    let client = connect_relay_config_client(service_address).await?;
    log::debug!("Connected to relay config service at {service_address}");

    request_ephemeral_peer_with(
        client,
        parent_pubkey,
        ephemeral_pubkey,
        enable_post_quantum,
        enable_daita,
    )
    .await
}

pub async fn request_ephemeral_peer_with(
    mut client: RelayConfigService,
    parent_pubkey: PublicKey,
    ephemeral_pubkey: PublicKey,
    enable_quantum_resistant: bool,
    enable_daita: bool,
) -> Result<EphemeralPeer, Error> {
    let (pq_request, kem_keypairs) = if enable_quantum_resistant {
        let start = Instant::now();
        let (pq_request, kem_keypairs) = post_quantum_secrets();
        log::debug!(
            "Generated quantum-resistant key exchange material in {} ms",
            start.elapsed().as_millis()
        );
        (Some(pq_request), Some(kem_keypairs))
    } else {
        (None, None)
    };

    let response = client
        .register_peer_v1(proto::EphemeralPeerRequestV1 {
            wg_parent_pubkey: parent_pubkey.as_bytes().to_vec(),
            wg_ephemeral_peer_pubkey: ephemeral_pubkey.as_bytes().to_vec(),
            post_quantum: pq_request,
            daita: None,
            daita_v2: enable_daita.then(|| {
                let platform = get_platform();
                log::trace!("DAITA v2 platform: {platform:?}");
                proto::DaitaRequestV2 {
                    level: i32::from(proto::DaitaLevel::LevelDefault),
                    platform: i32::from(platform),
                    version: DAITA_VERSION,
                }
            }),
        })
        .await
        .map_err(|status| Error::GrpcError(Box::new(status)))?;

    let response = response.into_inner();

    let psk = if let Some((ml_kem_keypair, hqc_keypair)) = kem_keypairs {
        let ciphertexts = response
            .post_quantum
            .ok_or(Error::MissingCiphertexts)?
            .ciphertexts;

        // Unpack the ciphertexts into one per KEM without needing to access them by index.
        let [ml_kem_ciphertext, hqc_ciphertext] = <&[Vec<u8>; 2]>::try_from(ciphertexts.as_slice())
            .map_err(|_| Error::InvalidCiphertextCount {
                actual: ciphertexts.len(),
            })?;

        // Store the PSK data on the heap. So it can be passed around and then zeroized on drop
        // without being stored in a bunch of places on the stack.
        let mut psk = PresharedKey::from(Box::default());

        // Decapsulate ML-KEM and mix into PSK
        {
            let mut shared_secret = ml_kem_keypair.decapsulate(ml_kem_ciphertext)?;
            xor_assign(psk.as_bytes_mut(), &shared_secret);

            // The shared secret is sadly stored in an array on the stack. So we can't get any
            // guarantees that it's not copied around on the stack. The best we can do here
            // is to zero out the version we have and hope the compiler optimizes out copies.
            // https://github.com/Argyle-Software/kyber/issues/59
            shared_secret.zeroize();
        }
        // Decapsulate HQC and mix into PSK
        {
            let mut shared_secret = hqc_keypair.decapsulate(hqc_ciphertext)?;
            xor_assign(psk.as_bytes_mut(), &shared_secret);

            // The shared secret is sadly stored in an array on the stack. So we can't get any
            // guarantees that it's not copied around on the stack. The best we can do here
            // is to zero out the version we have and hope the compiler optimizes out copies.
            // https://github.com/Argyle-Software/kyber/issues/59
            shared_secret.zeroize();
        }

        Some(psk)
    } else {
        None
    };

    let daita = response.daita.map(parse_daita_response).transpose()?;
    if daita.is_none() && enable_daita {
        return Err(Error::MissingDaitaResponse);
    }
    Ok(EphemeralPeer { psk, daita })
}

fn parse_daita_response(daita: proto::DaitaResponseV2) -> Result<DaitaSettings, Error> {
    let max_decoy_frac = parse_daita_fraction("max_padding_frac", daita.max_padding_frac)?;
    let max_delay_frac = parse_daita_fraction("max_blocking_frac", daita.max_blocking_frac)?;

    let machines = daita
        .client_machines
        .into_iter()
        .map(|machine| machine.parse())
        .collect::<Result<Vec<_>, daita::Error>>()
        .map_err(|error| {
            // NOTE: The parsing logic in `maybenot` always return the `Error::Machine` variant.
            let reason = match error {
                daita::Error::PaddingLimit | daita::Error::BlockingLimit => {
                    "unknown reason".to_string()
                }
                daita::Error::Machine(reason) => reason,
            };
            Error::ParseMaybenotMachines { reason }
        })?;
    Ok(DaitaSettings {
        client_machines: machines,
        max_decoy_frac,
        max_delay_frac,
    })
}

fn parse_daita_fraction(field: &'static str, value: f64) -> Result<f64, Error> {
    if (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(Error::InvalidDaitaFraction { field, value })
    }
}

const fn get_platform() -> proto::DaitaPlatform {
    use proto::DaitaPlatform;
    const PLATFORM: DaitaPlatform = cfg_select! {
        target_os = "windows" => { DaitaPlatform::WindowsWgGo }
        target_os = "linux"   => { DaitaPlatform::LinuxWgGo }
        target_os = "macos"   => { DaitaPlatform::MacosWgGo }
        target_os = "android" => { DaitaPlatform::AndroidWgGo }
        target_os = "ios"     => { DaitaPlatform::IosWgGo }
    };
    PLATFORM
}

fn post_quantum_secrets() -> (PostQuantumRequestV1, (ml_kem::Keypair, hqc::Keypair)) {
    let ml_kem_keypair = ml_kem::keypair();
    let hqc_keypair = hqc::keypair();

    (
        proto::PostQuantumRequestV1 {
            kem_pubkeys: vec![
                proto::KemPubkeyV1 {
                    algorithm_name: ml_kem_keypair.algorithm_name().to_owned(),
                    key_data: ml_kem_keypair.encapsulation_key(),
                },
                proto::KemPubkeyV1 {
                    algorithm_name: hqc_keypair.algorithm_name().to_owned(),
                    key_data: hqc_keypair.encapsulation_key(),
                },
            ],
        },
        (ml_kem_keypair, hqc_keypair),
    )
}

/// Performs `dst = dst ^ src`.
fn xor_assign(dst: &mut [u8; 32], src: &[u8; 32]) {
    for (dst_byte, src_byte) in dst.iter_mut().zip(src.iter()) {
        *dst_byte ^= src_byte;
    }
}

/// Create a new `RelayConfigService` connected to the given IP.
/// On non-Windows platforms the connection is made with a socket where the MSS
/// value has been speficically lowered, to avoid MTU issues. See the `socket` module.
#[cfg(not(target_os = "ios"))]
async fn connect_relay_config_client(ip: Ipv4Addr) -> Result<RelayConfigService, Error> {
    use hyper_util::rt::tokio::TokioIo;

    let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
    let addr = SocketAddr::new(IpAddr::V4(ip), CONFIG_SERVICE_PORT);

    let connection = endpoint
        .connect_with_connector(service_fn(move |_| async move {
            let sock = socket::TcpSocket::new()?;
            let stream = sock.connect(addr).await?;
            let sniffer = socket_sniffer::SocketSniffer::new(stream);
            Ok::<_, std::io::Error>(TokioIo::new(sniffer))
        }))
        .await
        .map_err(Error::GrpcConnectError)?;

    Ok(RelayConfigService::new(connection))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn daita_response(max_padding_frac: f64, max_blocking_frac: f64) -> proto::DaitaResponseV2 {
        proto::DaitaResponseV2 {
            client_machines: vec![],
            max_padding_frac,
            max_blocking_frac,
        }
    }

    fn parse_daita_response_error(daita: proto::DaitaResponseV2) -> Error {
        match parse_daita_response(daita) {
            Ok(_) => panic!("expected DAITA response parsing to fail"),
            Err(error) => error,
        }
    }

    #[test]
    fn parse_daita_response_accepts_fraction_bounds() {
        for (max_padding_frac, max_blocking_frac) in [(0.0, 0.0), (1.0, 1.0), (0.25, 0.75)] {
            let settings =
                parse_daita_response(daita_response(max_padding_frac, max_blocking_frac)).unwrap();

            assert_eq!(settings.max_decoy_frac, max_padding_frac);
            assert_eq!(settings.max_delay_frac, max_blocking_frac);
        }
    }

    #[test]
    fn parse_daita_response_rejects_invalid_padding_fraction() {
        for invalid_fraction in [
            -f64::EPSILON,
            1.0 + f64::EPSILON,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let error = parse_daita_response_error(daita_response(invalid_fraction, 0.5));

            match error {
                Error::InvalidDaitaFraction { field, value } => {
                    assert_eq!(field, "max_padding_frac");
                    if invalid_fraction.is_nan() {
                        assert!(value.is_nan());
                    } else {
                        assert_eq!(value, invalid_fraction);
                    }
                }
                error => panic!("unexpected error: {error:?}"),
            }
        }
    }

    #[test]
    fn parse_daita_response_rejects_invalid_blocking_fraction() {
        for invalid_fraction in [
            -f64::EPSILON,
            1.0 + f64::EPSILON,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let error = parse_daita_response_error(daita_response(0.5, invalid_fraction));

            match error {
                Error::InvalidDaitaFraction { field, value } => {
                    assert_eq!(field, "max_blocking_frac");
                    if invalid_fraction.is_nan() {
                        assert!(value.is_nan());
                    } else {
                        assert_eq!(value, invalid_fraction);
                    }
                }
                error => panic!("unexpected error: {error:?}"),
            }
        }
    }

    #[test]
    fn post_quantum_secrets_uses_expected_kems_in_order() {
        let (request, (ml_kem_keypair, hqc_keypair)) = post_quantum_secrets();

        let [ml_kem_pubkey, hqc_pubkey] =
            <&[proto::KemPubkeyV1; 2]>::try_from(request.kem_pubkeys.as_slice()).unwrap();

        assert_eq!(ml_kem_pubkey.algorithm_name, "ML-KEM-1024");
        assert_eq!(ml_kem_pubkey.key_data, ml_kem_keypair.encapsulation_key());

        assert_eq!(hqc_pubkey.algorithm_name, "HQC-256");
        assert_eq!(hqc_pubkey.key_data, hqc_keypair.encapsulation_key());
    }

    #[test]
    fn xor_assign_mixes_each_byte() {
        let mut dst = [0u8; 32];
        let src = [0xffu8; 32];

        xor_assign(&mut dst, &src);
        assert_eq!(dst, [0xffu8; 32]);

        xor_assign(&mut dst, &src);
        assert_eq!(dst, [0u8; 32]);
    }

    #[test]
    fn xor_assign_matches_bytewise_xor() {
        let mut dst = [0x55u8; 32];
        let src = [0x33u8; 32];

        xor_assign(&mut dst, &src);

        assert_eq!(dst, [0x66u8; 32]);
    }
}
