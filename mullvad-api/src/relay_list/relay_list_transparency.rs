use crate::rest;

/// A byte array representing a Sha256 hash.
pub type Sha256Bytes = [u8; 32];

/// The relay list digest (Sha256 hash as a hex string).
pub type RelayListDigest = String;

/// Sigsum signature and digest+timestamp for the relay list.
#[derive(Debug)]
pub struct RelayListSignature {
    /// This is the data that was signed by the sigsum signature. Note that this is *not* the
    /// relay list, but rather a metadata object in JSON that contains the hash of the relay list
    /// that corresponds to this signature and the timestamp of when it was signed.
    pub unparsed_timestamp: String,

    /// The sigsum signature for the signed `data`.
    pub sigsum_signature: String,
}

impl RelayListSignature {
    /// The API does not return JSON but instead a custom plain text format we need to parse
    /// it manually.
    pub fn from_server_response(response: &str) -> Result<RelayListSignature, rest::Error> {
        let (data, signature) = response
            .split_once("\n\n")
            .map(|(data, signature)| (format!("{data}\n"), signature.to_owned()))
            .ok_or(rest::Error::SigsumDeserializeError)?;

        Ok(RelayListSignature {
            unparsed_timestamp: data,
            sigsum_signature: signature,
        })
    }
}
