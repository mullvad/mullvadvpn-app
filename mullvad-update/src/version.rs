//! This module is used to extract the latest versions out of a raw [format::Response] using a query
//! [VersionParameters]. It also contains additional logic for filtering and validating the raw
//! deserialized response.
//!
//! The main input here is [VersionParameters], and the main output is [VersionInfo].

use std::cmp::Ordering;

use anyhow::Context;
use itertools::Itertools;
use mullvad_version::PreStableType;

use crate::format::{self, Installer};

/// Query type for [VersionInfo]
#[derive(Debug)]
pub struct VersionParameters {
    /// Architecture to retrieve data for
    pub architecture: VersionArchitecture,
    /// Rollout threshold. Any version in the response below this threshold will be ignored
    pub rollout: Rollout,
    /// Lowest allowed `metadata_version` in the version data
    /// Typically the current version plus 1
    pub lowest_metadata_version: usize,
}

/// Rollout threshold. Any version in the response below this threshold will be ignored
pub type Rollout = f32;

/// Accept *any* version (rollout >= 0) when querying for app info.
pub const IGNORE: Rollout = 0.;

/// Accept only fully rolled out versions (rollout >= 1) when querying for app info.
pub const FULLY_ROLLED_OUT: Rollout = 1.;

/// Installer architecture
pub type VersionArchitecture = format::Architecture;

/// Version information derived from querying a [format::Response] using [VersionParameters]
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct VersionInfo {
    /// Stable version info
    pub stable: Version,
    /// Beta version info (if available and newer than `stable`).
    /// If latest stable version is newer, this will be `None`.
    pub beta: Option<Version>,
}

/// Contains information about a version for the current target
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
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

/// Helper used to lift the relevant installer out of the array in [format::Release]
#[derive(Clone)]
struct IntermediateVersion {
    version: mullvad_version::Version,
    changelog: String,
    installer: format::Installer,
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
            installers
                .into_iter()
                // Find installer for the requested architecture (assumed to be unique)
                .find(|installer| params.architecture == installer.architecture)
                // Map each artifact to a [IntermediateVersion]
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

/// TODO: Document
pub fn get_installers(
    mut releases: Vec<format::Release>,
) -> Vec<(mullvad_version::Version, Installer)> {
    // Sort releases by version
    releases.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    releases
        .into_iter()
        .flat_map(|release| {
            release
                .installers
                .into_iter()
                .map(move |installer| (release.version.clone(), installer))
        })
        .collect()
}

impl TryFrom<IntermediateVersion> for Version {
    type Error = anyhow::Error;

    fn try_from(version: IntermediateVersion) -> Result<Self, Self::Error> {
        // Convert hex checksum to bytes
        let sha256 = hex::decode(version.installer.sha256)
            .context("Invalid checksum hex")?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid checksum length"))?;

        Ok(Version {
            version: version.version,
            size: version.installer.size,
            urls: version.installer.urls,
            changelog: version.changelog,
            sha256,
        })
    }
}

#[cfg(test)]
mod test {
    use insta::assert_yaml_snapshot;

    use super::*;

    // These tests rely on `insta` for snapshot testing. If they fail due to snapshot assertions,
    // then most likely the snapshots need to be updated. The most convenient way to review
    // changes to, and update, snapshots is by running `cargo insta review`.

    /// Test version info response handler (rollout 1, x86)
    #[test]
    fn test_version_info_parser_x86() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(include_bytes!(
            "../test-version-response.json"
        ))?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: 1.,
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
        let response = format::SignedResponse::deserialize_insecure(include_bytes!(
            "../test-version-response.json"
        ))?;

        let params = VersionParameters {
            architecture: VersionArchitecture::Arm64,
            rollout: 0.01,
            lowest_metadata_version: 0,
        };

        let info = VersionInfo::try_from_response(&params, response.signed)?;

        // Expect: The available latest versions for arm64, where the rollout is .01.
        assert_yaml_snapshot!(info);

        Ok(())
    }
}
