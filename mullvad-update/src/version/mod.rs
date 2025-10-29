//! This module is used to extract the latest versions out of a raw [format::Response] using a query
//! [VersionParameters]. It also contains additional logic for filtering and validating the raw
//! deserialized response.
//!
//! The main input here is [VersionParameters], and the main output is [VersionInfo].

pub mod rollout;

use std::{
    cmp::Ordering,
    fmt::{self, Display},
    ops::RangeInclusive,
    str::FromStr,
};

use anyhow::{Context, bail};
use itertools::Itertools;
use mullvad_version::PreStableType;
use serde::{Deserialize, Serialize, de::Error};

use crate::format::{self, Installer, Response};
use rollout::Rollout;

/// Lowest version to accept using 'verify'
pub const MIN_VERIFY_METADATA_VERSION: usize = 0;

/// Query type for [VersionInfo]
#[derive(Debug, Clone)]
pub struct VersionParameters {
    /// Architecture to retrieve data for
    pub architecture: VersionArchitecture,
    /// Rollout threshold. Any version in the response below this threshold will be ignored
    pub rollout: Rollout,
    /// Allow versions without any installers to be returned
    pub allow_empty: bool,
    /// Lowest allowed `metadata_version` in the version data
    /// Typically the current version plus 1
    pub lowest_metadata_version: usize,
}

/// Accept *any* version (rollout >= 0) when querying for app info.
pub const IGNORE: Rollout = Rollout(0.);

/// Accept any version (rollout > 0) when querying for app info.
/// Only versions with a non-zero rollout are supported.
pub const SUPPORTED_VERSION: Rollout = Rollout(f32::EPSILON);

/// Accept only fully rolled out versions (rollout >= 1) when querying for app info.
pub const FULLY_ROLLED_OUT: Rollout = Rollout(1.);

pub const VALID_ROLLOUT: RangeInclusive<f32> = 0.0..=1.0;

/// Installer architecture
pub type VersionArchitecture = format::Architecture;

/// Version update information derived from querying a [format::Response] and filtering with [VersionParameters]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct VersionInfo {
    /// Stable version info
    pub stable: Version,
    /// Beta version info (if available and newer than `stable`).
    /// If latest stable version is newer, this will be `None`.
    pub beta: Option<Version>,
}

/// Contains information about a version for the current target
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Version {
    /// Version
    pub version: mullvad_version::Version,
    /// URLs to use for downloading the app installer
    pub urls: Vec<String>,
    /// Size of installer, in bytes
    pub size: usize,
    /// Version changelog
    pub changelog: String,
    /// App installer checksum
    pub sha256: [u8; 32],
}

impl VersionInfo {
    /// Convert signed response data to public version type
    /// NOTE: `response` is assumed to be verified and untampered. It is not verified.
    pub fn try_from_response(
        params: &VersionParameters,
        response: format::Response,
    ) -> anyhow::Result<Self> {
        // Fail if there are duplicate versions.
        // Check this before anything else so that it's rejected independently of `params`.
        if !response.releases.iter().map(|r| &r.version).all_unique() {
            anyhow::bail!("API response contains multiple release for the same version");
        }

        // Filter releases based on rollout, architecture and dev versions.
        let available_versions: Vec<Version> = response.releases
        .into_iter()
        // Filter out releases that are not rolled out to us
        .filter(|release| release.rollout >= params.rollout)
        // Filter out dev versions
        .filter(|release| !release.version.is_dev())
        .flat_map(|format::Release { version, changelog, installers, .. }| {
            if installers.is_empty() && params.allow_empty {
                // HACK: If there are no installers (e.g. on Linux), return the version anyway
                return Some(anyhow::Ok(Version {
                    version,
                    size: 0,
                    urls: vec![],
                    changelog,
                    sha256: [0u8; 32],
                }));
            }
            installers
                .into_iter()
                // Find installer for the requested architecture (assumed to be unique)
                .find(|installer| params.architecture == installer.architecture)
                // Map each artifact to a [Version]
                .map(|Installer { urls, size, sha256,.. }| {
                    anyhow::Ok(Version {
                        version,
                        size,
                        urls,
                        changelog,
                        sha256: hex::decode(sha256)
                        .context("Invalid checksum hex")?
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid checksum length"))?,
                    })
                })
        }).try_collect()?;

        // Find latest stable version
        let stable = available_versions
            .iter()
            .filter(|release| release.version.pre_stable.is_none())
            .max_by(|a, b| a.version.partial_cmp(&b.version).unwrap_or(Ordering::Equal))
            .context("No stable version found")?
            .clone();

        // Find the latest beta version
        let beta = available_versions
            .iter()
            .filter(|release| matches!(release.version.pre_stable, Some(PreStableType::Beta(_))))
            // If the latest beta version is older than latest stable, dispose of it
            .filter(|release| release.version > stable.version)
            .max_by(|a, b| a.version.partial_cmp(&b.version).unwrap_or(Ordering::Equal))
            .cloned();

        Ok(Self { stable, beta })
    }
}

/// A version is considered supported if the version exists in the metadata. Versions with a
/// rollout of 0 is still considered supported.
pub fn is_version_supported(
    current_version: mullvad_version::Version,
    response: &Response,
) -> bool {
    response
        .releases
        .iter()
        .any(|release| release.version.eq(&current_version))
}

impl Rollout {
    /// Calculate the threshold used to determine if a client is included in the current rollout of
    /// some release.
    ///
    /// Invariant: 0.0 < threshold <= 1.0
    ///
    /// 0.0 is a special-cased rollout value reserved for complete rollbacks. See [IGNORE].
    pub fn threshold(rollout_threshold_seed: u32, version: mullvad_version::Version) -> Self {
        use rand::{Rng, SeedableRng, rngs::SmallRng};
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(rollout_threshold_seed.to_string());
        hasher.update(version.to_string());
        let hash = hasher.finalize();
        let seed: &[u8; 32] = hash.first_chunk().expect("SHA256 hash is 32 bytes");
        let mut rng = SmallRng::from_seed(*seed);
        let threshold = rng.random_range(SUPPORTED_VERSION.0..=FULLY_ROLLED_OUT.0);
        Self::try_from(threshold).expect("threshold is within the Rollout domain")
    }
}

impl TryFrom<f32> for Rollout {
    type Error = anyhow::Error;

    fn try_from(rollout: f32) -> Result<Self, Self::Error> {
        if !rollout.is_finite() {
            bail!("rollout value must be a finite number, but was {rollout}");
        }

        if !VALID_ROLLOUT.contains(&rollout) {
            bail!(
                "rollout value {rollout} is outside valid range {}..={}",
                VALID_ROLLOUT.start(),
                VALID_ROLLOUT.end(),
            );
        }

        Ok(Rollout(rollout))
    }
}

impl Eq for Rollout {}

#[allow(clippy::derive_ord_xor_partial_ord)] // we impl Ord in terms of PartalOrd, so it's fine
impl Ord for Rollout {
    fn cmp(&self, other: &Self) -> Ordering {
        debug_assert!(self.0.is_finite());
        debug_assert!(other.0.is_finite());
        self.partial_cmp(other).expect("rollout is always in 0..=1")
    }
}

impl Display for Rollout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Rollout {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rollout: f32 = s.parse()?;
        Rollout::try_from(rollout)
    }
}

impl<'de> Deserialize<'de> for Rollout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let rollout = f32::deserialize(deserializer)?;

        Rollout::try_from(rollout)
            .map_err(|e| e.to_string())
            .map_err(D::Error::custom)
    }
}

impl Serialize for Rollout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Generate a special seed used to calculate at which rollout percentage a client should be
/// notified about a new release.
///
/// See [Rollout::threshold] for details.
pub fn generate_rollout_seed() -> u32 {
    rand::random()
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use insta::{assert_snapshot, assert_yaml_snapshot};

    use super::*;

    const TEST_RESPONSE: &[u8] = include_bytes!("../../test-version-response.json");

    // These tests rely on `insta` for snapshot testing. If they fail due to snapshot assertions,
    // then most likely the snapshots need to be updated. The most convenient way to review
    // changes to, and update, snapshots is by running `cargo insta review`.

    /// Test version info response handler (rollout 1, x86)
    #[test]
    fn test_version_info_parser_x86() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE)?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: FULLY_ROLLED_OUT,
            allow_empty: false,
            lowest_metadata_version: 0,
        };

        // Expect: The available latest versions for X86, where the rollout is 1.
        let info = VersionInfo::try_from_response(&params, response.signed.clone())?;

        assert_yaml_snapshot!(info);

        Ok(())
    }

    /// Test version info response handler (rollout 0.01, arm64)
    #[test]
    fn test_version_info_parser_arm64() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE)?;

        let params = VersionParameters {
            architecture: VersionArchitecture::Arm64,
            rollout: SUPPORTED_VERSION,
            allow_empty: false,
            lowest_metadata_version: 0,
        };

        let info = VersionInfo::try_from_response(&params, response.signed)?;

        // Expect: The available latest versions for arm64, where the rollout is .01.
        assert_yaml_snapshot!(info);

        Ok(())
    }

    /// Versions without installers should be returned if `allow_empty` is set
    #[test]
    fn test_version_info_empty() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE)?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: SUPPORTED_VERSION,
            allow_empty: true,
            lowest_metadata_version: 0,
        };

        let info = VersionInfo::try_from_response(&params, response.signed)?;

        // Expect: The available latest versions for x86, where the rollout is .01.
        assert_yaml_snapshot!(info);

        Ok(())
    }

    /// Test whether [SUPPORTED_VERSION] ignores unsupported versions (where rollout = 0.0)
    #[test]
    fn test_version_unsupported_filtering() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE)?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: SUPPORTED_VERSION,
            allow_empty: true,
            lowest_metadata_version: 0,
        };

        let info = VersionInfo::try_from_response(&params, response.signed.clone())?;

        // Expect: The available latest versions for x86, where the rollout is non-zero.
        assert_yaml_snapshot!(info);

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: IGNORE,
            allow_empty: true,
            lowest_metadata_version: 0,
        };

        let info = VersionInfo::try_from_response(&params, response.signed)?;

        // Expect: There is an even higher version where the rollout is zero.
        assert_yaml_snapshot!(info);

        Ok(())
    }

    #[test]
    fn test_is_version_supported() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE)?;

        let supported_version = mullvad_version::Version::from_str("2025.3").unwrap();
        let supported_rollout_zero_version = mullvad_version::Version::from_str("2030.3").unwrap();
        let non_supported_version = mullvad_version::Version::from_str("2025.5").unwrap();

        assert!(is_version_supported(supported_version, &response.signed));
        assert!(is_version_supported(
            supported_rollout_zero_version,
            &response.signed
        ));
        assert!(!is_version_supported(
            non_supported_version,
            &response.signed
        ));

        Ok(())
    }

    const GOOD_ROLLOUT_EXAMPLES: &[f32] = &[
        -0.0,                // 0%
        0.0,                 // 0%
        -0.0 + f32::EPSILON, // > 0%
        1.0 / 3.0,           // 33%
        1.0 - f32::EPSILON,  // 99%
        1.0,                 // 100%
    ];

    const BAD_ROLLOUT_EXAMPLES: &[f32] = &[
        -f32::EPSILON,
        1.0 + f32::EPSILON,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        100.0,
    ];

    #[test]
    fn test_rollout_serialization() {
        for &valid_rollout in GOOD_ROLLOUT_EXAMPLES {
            let serialized_f32 = serde_json::to_string(&valid_rollout).unwrap();
            let deserialized_rollout: Rollout = serde_json::from_str(&serialized_f32).unwrap();
            let serialized_rollout = serde_json::to_string(&deserialized_rollout).unwrap();

            assert_eq!(deserialized_rollout.0, valid_rollout);
            assert_eq!(serialized_rollout, serialized_f32);
        }
    }

    #[test]
    fn test_rollout_deserialize_bad() {
        for &bad_rollout in BAD_ROLLOUT_EXAMPLES {
            let rollout_str = bad_rollout.to_string();
            serde_json::from_str::<Rollout>(&rollout_str)
                .expect_err("must fail to deserialize bad rollout");
        }
    }

    /// Test that the `Display` impl of [Rollout] makes sense.
    /// Note clap requires that `Display` must be the inverse of `FromStr`.
    #[test]
    fn test_rollout_display() {
        let string_reprs = GOOD_ROLLOUT_EXAMPLES
            .iter()
            .map(|&f| format!("{f} => {}\n", Rollout::try_from(f).unwrap()))
            .collect::<String>();

        assert_snapshot!(&string_reprs);
    }

    #[test]
    /// Check that the implementation of [rollout_threshold] yields different threshold values as
    /// app version number progresses.
    ///
    /// Note that there is a chance for repetition - we are effectively mapping a 256 byte hash to
    /// the fractional part of an [f32], which is a much smaller domain.
    fn test_rollout_threshold_uniqueness() {
        let seed = 4; // Chosen by fair dice roll. Guaranteed to be random.
        let v20254: mullvad_version::Version = "2025.4".parse().unwrap();
        let v20255: mullvad_version::Version = "2025.5".parse().unwrap();
        assert_ne!(
            Rollout::threshold(seed, v20254.clone()),
            Rollout::threshold(seed, v20255.clone())
        );
        assert_yaml_snapshot!(Rollout::threshold(seed, v20254));
        assert_yaml_snapshot!(Rollout::threshold(seed, v20255));
    }
}
