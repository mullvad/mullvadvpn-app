use crate::relay_list_transparency::validate;
use crate::{CachedRelayList, RelayListProxy, SigsumPublicKey, rest};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// A byte array representing a Sha256 hash.
pub type Sha256Bytes = [u8; 32];

/// The relay list digest (Sha256 hash as a hex string).
pub type RelayListDigest = String;

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

/// Downloads and verifies the transparency logged relay list.
/// If the verification fails the error is only logged, and a new relay list will still be
/// fetched and used as long as we are able to parse the digest (which is needed to fetch
/// the relay list).
pub(crate) async fn download_and_verify_relay_list(
    proxy: &RelayListProxy,
    latest_digest: Option<RelayListDigest>,
    latest_timestamp: Option<DateTime<Utc>>,
    sigsum_trusted_pubkeys: Vec<SigsumPublicKey>,
) -> Result<Option<CachedRelayList>, rest::Error> {
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
        log::debug!("SIGSUM: timestamp digest hasn't changed; will not fetch new relay list");
        return Ok(None);
    }

    // Fetch the actual relay list given the timestamp digest.
    let response = proxy
        .relay_list_content(&timestamp.digest, timestamp.timestamp)
        .await?;

    // Validate that the sigsum digest matches the relay list hash.
    match validate::validate_relay_list_content(&timestamp, response.digest()) {
        Ok(_) => log::debug!("SIGSUM: Relay list sigsum data validation succeeded"),
        Err(e) => log::error!("SIGSUM: Relay list sigsum data validation failed: {}", e),
    }

    Ok(Some(response))
}
