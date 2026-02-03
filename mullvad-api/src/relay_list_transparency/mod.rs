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
    pub fn new(digest: String) -> Self {
        RelayListDigest(digest)
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

/// The unparsed Sigsum signature and digest+timestamp for the relay list as returned by the API.
#[derive(Debug)]
pub struct RelayListSignature {
    /// This is the data that was signed by the sigsum signature. Note that this is *not* the
    /// relay list, but rather a metadata object in JSON that contains the hash of the relay list
    /// that corresponds to this signature and the timestamp of when it was signed.
    pub unparsed_timestamp: String,

    /// The sigsum signature for the signed `unparsed_timestamp`.
    pub unparsed_sigsum_signature: String,
}

impl RelayListSignature {
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
    pub fn parse(response: &str) -> Result<RelayListSignature, rest::Error> {
        let (data, signature) = response
            .split_once("\n\n")
            .map(|(data, signature)| (format!("{data}\n"), signature.to_owned()))
            .ok_or(rest::Error::SigsumDeserializeError)?;

        Ok(RelayListSignature {
            unparsed_timestamp: data,
            unparsed_sigsum_signature: signature,
        })
    }
}

/// The unparsed relay list and content digest and timestamp.
#[derive(Debug)]
pub struct RelayListContent {
    /// The unparsed relay list JSON as raw bytes.
    pub content: Vec<u8>,
    /// The digest that for the raw JSON bytes.
    pub digest: RelayListDigest,
    /// The timestamp that was returned from the corresponding call to `/trl/v0/timestamps/latest`.
    pub timestamp: DateTime<Utc>,
}

/// Downloads and verifies the transparency logged relay list.
/// If the verification fails the error is only logged, and a new relay list will still be
/// fetched and used as long as we are able to parse the digest (which is needed to fetch
/// the relay list).
///
/// * `latest_digest` - The latest digest that the app has successfully fetched.
///   If the digest we get from the API is the same as `latest_digest` we do not need to
///   download the relay list again. If `None` the relay list is fetched unconditionally.
/// * `latest_timestamp` - The latest timestamp that the app has successfully fetched.
///   This is used to verify that the next timestamp we get isn't too old.
/// * `sigsum_trusted_pubkeys` - The sigsum pubkeys that should be used for verification.
pub(crate) async fn download_and_verify_relay_list(
    proxy: &RelayListProxy,
    latest_digest: Option<RelayListDigest>,
    latest_timestamp: Option<DateTime<Utc>>,
    sigsum_trusted_pubkeys: &[SigsumPublicKey],
) -> Result<Option<RelayListContent>, rest::Error> {
    // Fetch relay list latest sigsum signature.
    let relay_list_sig = proxy.relay_list_latest_timestamp().await?;

    // Parse the timestamp from the signature.
    let timestamp =
        match validate::validate_relay_list_signature(&relay_list_sig, sigsum_trusted_pubkeys) {
            Ok(timestamp) => {
                log::debug!("SIGSUM: Relay list sigsum signature validation succeeded");
                timestamp
            }
            Err(e) => {
                log::error!(
                    "SIGSUM: Relay list sigsum signature validation failed: {}",
                    e.source
                );
                log::debug!("SIGSUM: Attempting to parse unverified timestamp");

                e.timestamp_parser
                    .parse_without_verification()
                    .inspect_err(|_| {
                        log::error!(
                        "SIGSUM: Failed to parse unverified timestamp; aborting relay list update"
                    );
                    })
                    .inspect(|_| log::debug!("SIGSUM: Successfully parsed unverified timestamp"))
                    .map_err(rest::Error::from)?
            }
        };

    // Verify that the timestamp is not too old.
    let new_timestamp = timestamp.timestamp;
    if new_timestamp < (Utc::now() - Duration::from_hours(24)) {
        log::error!("SIGSUM: Relay list timestamp is older than 24 hours: {new_timestamp}",);
    }

    // Verify that the timestamp we got from the API is not older than the most recent timestamp
    // we have seen.
    if let Some(ts) = latest_timestamp
        && new_timestamp < ts
    {
        log::error!(
            "SIGSUM: Relay list timestamp is older than current timestamp\n\
                current {ts}, new: {new_timestamp}",
        );
    }

    // If the digest has not changed we do not need to fetch the relay list.
    if let Some(digest) = latest_digest
        && digest == timestamp.digest
    {
        log::debug!("SIGSUM: timestamp digest hasn't changed - will not fetch new relay list");
        return Ok(None);
    }

    // Fetch the actual relay list given the timestamp digest.
    let relay_list_response = proxy.relay_list_content(&timestamp.digest).await?;

    // Validate that the sigsum digest matches the relay list hash.
    match validate::validate_relay_list_content(&timestamp, &relay_list_response.digest) {
        Ok(_) => log::debug!("SIGSUM: Relay list sigsum data validation succeeded"),
        Err(e) => log::error!("SIGSUM: Relay list sigsum data validation failed: {}", e),
    }

    Ok(Some(RelayListContent {
        content: relay_list_response.content,
        digest: relay_list_response.digest,
        timestamp: new_timestamp,
    }))
}
