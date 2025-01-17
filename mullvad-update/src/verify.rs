use std::{fs::File, io::BufReader, path::Path};

use anyhow::Context;
use pgp::{
    armor,
    packet::{Packet, PacketParser},
    types::PublicKeyTrait,
    Deserializable, SignedPublicKey,
};

/// A verifier of digital file signatures
pub trait AppVerifier: 'static + Clone {
    /// Verify `bin_path` using the signature at `sig_path`, and return an error if this fails for
    /// any reason.
    fn verify(bin_path: impl AsRef<Path>, sig_path: impl AsRef<Path>) -> anyhow::Result<()>;
}

/// Verification using pgp
#[derive(Clone)]
pub struct PgpVerifier;

impl PgpVerifier {
    const SIGNING_PUBKEY: &[u8] = include_bytes!("../mullvad-code-signing.gpg");
}

impl AppVerifier for PgpVerifier {
    fn verify(bin_path: impl AsRef<Path>, sig_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let pubkey = SignedPublicKey::from_bytes(Self::SIGNING_PUBKEY)?;

        let sig_reader = BufReader::new(File::open(sig_path).context("Open signature file")?);
        let signature = PacketParser::new(armor::Dearmor::new(sig_reader))
            .find_map(|packet| {
                if let Ok(Packet::Signature(sig)) = packet {
                    Some(sig)
                } else {
                    None
                }
            })
            .context("Missing signature")?;
        let issuer = signature
            .issuer()
            .into_iter()
            .next()
            .context("Find issuer key ID")?;

        // Find subkey used for signing
        let subkey = pubkey
            .public_subkeys
            .iter()
            .find(|subkey| &subkey.key_id() == issuer)
            .context("Find signing subkey")?;
        //subkey.verify(&pubkey)?;

        let bin = BufReader::with_capacity(1024 * 1024, File::open(bin_path)?);

        signature
            .verify(subkey, bin)
            .context("Verification failed")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pgp_signing_pubkey() {
        SignedPublicKey::from_bytes(PgpVerifier::SIGNING_PUBKEY).unwrap();
    }
}
