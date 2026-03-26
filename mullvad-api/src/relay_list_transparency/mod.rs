//! This module contains types and functions for parsing and validating the Sigsum transparency log
//! signatures and timestamps that are available at the `/trl/v0/timestamps/latest` and
//! `/trl/v0/data/` endpoints.

mod test;
mod validate;

use crate::{RelayListProxy, SigsumPublicKey, rest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;
pub use validate::{SigsumPublicKeyParseError, parse_pubkeys};

/// A byte array representing a Sha256 hash.
pub type Sha256Bytes = [u8; 32];

/// The relay list digest (Sha256 hash as a hex string).
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct RelayListDigest(String);

impl RelayListDigest {
    pub fn new(digest: Sha256Bytes) -> Self {
        RelayListDigest(hex::encode(digest))
    }
}

impl AsRef<str> for RelayListDigest {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for RelayListDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The relay list digest together with its timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct SigsumPayload {
    /// The digest of the relay list.
    pub digest: RelayListDigest,

    /// When the signature was signed.
    pub timestamp: DateTime<Utc>,
}

impl SigsumPayload {
    pub fn new(digest: RelayListDigest, timestamp: DateTime<Utc>) -> Self {
        SigsumPayload { digest, timestamp }
    }
}

/// The unparsed Sigsum signature and digest+timestamp for the relay list as returned by the API.
#[derive(Debug)]
pub struct RelayListEnvelope {
    /// This is the data that was signed by the sigsum signature. Note that this is *not* the
    /// relay list, but rather a metadata object in JSON that contains the hash of the relay list
    /// that corresponds to this signature and the timestamp of when it was signed.
    pub unparsed_payload: String,

    /// The sigsum signature for the signed `unparsed_payload`.
    pub unparsed_signature: String,
}

impl RelayListEnvelope {
    /// The API does not return JSON but instead a custom plain text format we need to parse
    /// it manually.
    /// The format is:
    ///
    /// {timestamp and digest data encoded as newline-delimited JSON}\n
    /// \n
    /// {the sigsum signature}
    ///
    /// Note that the first newline after the JSON line is significant and part of the data that was
    /// signed by the sigsum signature.
    pub fn parse(response: &str) -> Result<RelayListEnvelope, rest::Error> {
        let (unparsed_data, unparsed_signature) = response
            .split_once("\n\n")
            .map(|(data, signature)| (format!("{data}\n"), signature.to_owned()))
            .ok_or(rest::Error::SigsumDeserializeError)?;

        Ok(RelayListEnvelope {
            unparsed_payload: unparsed_data,
            unparsed_signature,
        })
    }
}

/// The unparsed relay list and content digest and timestamp.
#[derive(Debug)]
pub struct SigsumVerifiedRelayList {
    /// The unparsed relay list JSON as raw bytes.
    pub content: Vec<u8>,
    /// The digest for the raw JSON bytes.
    pub digest: RelayListDigest,
    /// The timestamp that was returned from the corresponding call to `/trl/v0/timestamps/latest`.
    pub timestamp: DateTime<Utc>,
}

/// The timestamp must not be older than this for it to be valid.
static TIMESTAMP_MAX_VALID_AGE: Duration = Duration::from_hours(24);

/// Downloads and verifies the transparency logged relay list.
/// If the verification fails the error is only logged, and a new relay list will still be
/// fetched and used as long as we are able to parse the digest (which is needed to fetch
/// the relay list).
///
/// * `latest_digest` - The latest digest that the app has successfully fetched.
///   If the digest we get from the API is the same as `latest_digest` we do not need to
///   download the relay list again. If `None` the relay list is fetched unconditionally.
/// * `sigsum_trusted_pubkeys` - The sigsum pubkeys that should be used for verification.
pub async fn download_and_verify_relay_list(
    proxy: &RelayListProxy,
    current_digest: Option<SigsumPayload>,
    sigsum_trusted_pubkeys: &[SigsumPublicKey],
) -> Result<Option<SigsumVerifiedRelayList>, rest::Error> {
    // Fetch relay list latest sigsum envelope.
    let envelope = proxy.relay_list_latest_envelope().await?;

    // Parse the payload from the envelope.
    let payload = match validate::validate_relay_list_envelope(&envelope, sigsum_trusted_pubkeys) {
        Ok(payload) => {
            log::debug!("SIGSUM: Relay list sigsum signature validation succeeded");
            payload
        }
        Err(e) => {
            log::error!(
                "SIGSUM: Relay list sigsum signature validation failed: {}",
                e.source
            );
            log::debug!("SIGSUM: Attempting to parse unverified payload");

            e.timestamp_parser
                .parse_without_verification()
                .inspect_err(|_| {
                    log::error!(
                        "SIGSUM: Failed to parse unverified payload; aborting relay list update"
                    );
                })
                .inspect(|_| log::debug!("SIGSUM: Successfully parsed unverified payload"))
                .map_err(rest::Error::from)?
        }
    };

    // Verify that the timestamp is not too old.
    let new_timestamp = payload.timestamp;
    if new_timestamp < (Utc::now() - TIMESTAMP_MAX_VALID_AGE) {
        log::error!("SIGSUM: Relay list timestamp is too old: {new_timestamp}",);
    }

    if let Some(current) = current_digest {
        // Verify that the timestamp we got from the API is not older than the most recent timestamp
        // we have seen.
        if new_timestamp < current.timestamp {
            log::error!(
                "SIGSUM: Relay list timestamp is older than current timestamp\n\
                current {}, new: {new_timestamp}",
                current.timestamp,
            );
        }

        // If the digest has not changed we do not need to fetch the relay list.
        if current.digest == payload.digest {
            log::debug!("SIGSUM: Payload digest hasn't changed - will not fetch new relay list");
            return Ok(None);
        }
    }

    // Fetch the actual relay list given the payload digest.
    let relay_list_response = proxy.relay_list_content(&payload.digest).await?;

    // Validate that the fetched relay list digest matches the sigsum digest.
    if relay_list_response.digest == payload.digest {
        log::debug!("SIGSUM: Relay list sigsum data validation succeeded");
    } else {
        log::error!("SIGSUM: Fetched relay list digest does not equal sigsum digest");
    }

    Ok(Some(SigsumVerifiedRelayList {
        content: relay_list_response.content,
        digest: relay_list_response.digest,
        timestamp: new_timestamp,
    }))
}
