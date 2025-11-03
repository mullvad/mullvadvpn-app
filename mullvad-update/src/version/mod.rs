//! This module is used to extract the latest versions out of a raw [format::Response] using a query
//! [VersionParameters]. It also contains additional logic for filtering and validating the raw
//! deserialized response.
//!
//! The main input here is [VersionParameters], and the main output is [VersionInfo].

pub mod rollout;

use std::cmp::Ordering;

use anyhow::{Context, bail};
use itertools::Itertools;

use crate::format::{self, Installer as AppInstaller, Release};
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
    /// App installer
    #[serde(flatten)]
    installer: Installer,
    /// Version changelog
    pub changelog: String,
}

impl Version {
    /// Create a [Version] which does not contain an installer.
    ///
    /// This is useful for broadcasting version info on Linux, where installers will never exist.
    pub fn suckless(version: mullvad_version::Version, changelog: String) -> Self {
        Version {
            version,
            changelog,
            installer: Installer::empty(),
        }
    }

    /// Returns the chronological order between two app versions.
    ///
    /// Invariant: the versions compared ought to be of the same kind, i.e. stable, beta or alpha.
    // F: FnMut(&Self::Item, &Self::Item) -> Ordering
    pub fn latest(&self, other: &Self) -> Ordering {
        self.version
            .partial_cmp(&other.version)
            .unwrap_or(Ordering::Equal)
    }

    /// True if self is a stable app version.
    pub const fn stable(&self) -> bool {
        self.version.is_stable()
    }

    /// True if self is a beta app version.
    pub const fn beta(&self) -> bool {
        self.version.is_beta()
    }
}

/// TODO: Document
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
struct Installer {
    /// URLs to use for downloading the app installer
    urls: Vec<String>,
    /// Size of installer, in bytes
    size: usize,
    /// App installer checksum
    sha256: [u8; 32],
}

impl Installer {
    fn new(urls: Vec<String>, size: usize, sha256: String) -> anyhow::Result<Self> {
        Ok(Self {
            size,
            urls,
            sha256: hex::decode(sha256)
                .context("Invalid checksum hex")?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid checksum length"))?,
        })
    }

    fn empty() -> Self {
        Self::default()
    }
}

impl VersionInfo {
    /// Convert signed response data to public version type
    /// NOTE: `response` is assumed to be verified and untampered. It is not verified.
    //
    // TODO: Decompose
    pub fn try_from_response(
        params: &VersionParameters,
        response: format::Response,
    ) -> anyhow::Result<Self> {
        // Check this before anything else so that it's rejected independently of `params`.
        Self::validate_releases(&response.releases)?;

        let version_info = get_version_info(
            response.releases,
            params.rollout,
            params.allow_empty,
            params.architecture,
        )?;

        Ok(version_info)
    }

    /// Fail if there are duplicate versions.
    fn validate_releases(releases: &[Release]) -> anyhow::Result<()> {
        if !releases.iter().map(|r| &r.version).all_unique() {
            bail!("API response contains multiple release for the same version");
        }
        Ok(())
    }
}

/// TODO: Document this function *very well*.
/// Input: TODO
/// Output: TODO
fn get_version_info(
    releases: Vec<Release>,
    rollout: Rollout,
    allow_empty: bool,
    architecture: format::Architecture,
) -> anyhow::Result<VersionInfo> {
    let available_versions: Vec<Version> =
        get_version_info_intermediary(releases, rollout, allow_empty, architecture)?;
    let stable = get_latest_version(&available_versions, Version::stable)
        .context("No stable version found")?;
    // Find the latest beta version.
    //
    // If the latest beta version is older than latest stable, dispose of it.
    let beta = get_latest_version(&available_versions, |app| {
        app.beta() && app.version > stable.version
    });
    Ok(VersionInfo {
        stable: stable.clone(),
        beta: beta.cloned(),
    })
}

/// Filter releases based on rollout, architecture and dev versions.
///
/// Properties:
/// - If no release in `releases`
fn get_version_info_intermediary(
    releases: Vec<Release>,
    rollout: Rollout,
    allow_empty: bool,
    architecture: format::Architecture,
) -> anyhow::Result<Vec<Version>> {
    // Map a [Release] into an [AppInstaller]
    let something = |release: Release| {
        // TODO: Can this edge-case be replaced?
        if release.installers().is_empty() && allow_empty {
            return Some(Ok(Version::suckless(release.version, release.changelog)));
        }
        // Map each artifact to a [Version]
        let version = release.version.clone();
        let changelog = release.changelog.clone();
        release.into_installer(architecture).map(
            |AppInstaller {
                 urls, size, sha256, ..
             }| {
                let installer = Installer::new(urls, size, sha256)?;
                Ok(Version {
                    version,
                    changelog,
                    installer,
                })
            },
        )
    };

    // Filter releases based on rollout, architecture and dev versions.
    releases
        .into_iter()
        // Filter out releases that are not rolled out to us
        .filter(|release| release.rollout >= rollout)
        // Filter out dev versions
        .filter(|release| !release.version.is_dev())
        .flat_map(something).try_collect()
}

/// Find latest version of some kind (i.e. stable, beta, alpha).
//
// TODO: Define properties of this function.
fn get_latest_version(
    available_versions: &[Version],
    kind: impl Fn(&Version) -> bool,
) -> Option<&Version> {
    available_versions
        .iter()
        .filter(|v| kind(v))
        .max_by(|this, that| Version::latest(this, that))
}

/// A version is considered supported if the version exists in the metadata. Versions with a
/// rollout of 0 is still considered supported.
// TODO: Define properties
pub fn is_version_supported(
    current_version: mullvad_version::Version,
    releases: &[Release],
) -> bool {
    releases
        .iter()
        .any(|release| release.version.eq(&current_version))
}

/// Generate a special seed used to calculate at which rollout percentage a client should be
/// notified about a new release.
///
/// See [Rollout::threshold] for details.
pub fn generate_rollout_seed() -> u32 {
    rand::random()
}

#[cfg(test)]
pub mod arbitrary {
    use super::*;

    use format::Architecture;
    use mullvad_version::arbitrary::arb_version as arb_app_version;
    use prop::collection;
    use prop::option;
    use proptest::prelude::*;
    use rollout::arbitrary::*;

    prop_compose! {
        /// Generate arbitrary [VersionParameters].
        pub fn arb_version_parameters(allow_empty: bool)
            (rollout in arb_rollout(), architecture in arb_architecture(), lowest_metadata_version in 0..100usize)
            -> VersionParameters {
                // TODO: Check the `lowest_metadata_version` generations.
            VersionParameters { architecture, rollout, allow_empty, lowest_metadata_version }
        }
    }

    prop_compose! {
        /// Generate arbitrary [VersionInfo] (API responses).
        pub fn arb_version_info()
            (stable in arb_version(), beta in option::of(arb_version()))
            -> VersionInfo {
            VersionInfo { stable, beta }
        }
    }

    prop_compose! {
        /// Generate arbitrary [Version].
        pub fn arb_version()
                          (app_version in arb_app_version(), changelog: String, installer in arb_installer())
                          -> Version {
            Version {
                version: app_version,
                installer,
                changelog,
            }
        }
    }

    prop_compose! {
        /// Generate arbitrary [Version] with empty installers.
        pub fn arb_version_empty_installer()
               (app_version in arb_app_version(), changelog: String, installer in empty_installer())
               -> Version {
            Version {
                version: app_version,
                installer,
                changelog,
            }
        }
    }

    /// Generate an arbitrary [Installer].
    // TODO: This sucks.
    fn arb_installer() -> impl Strategy<Value = Installer> {
        // TODO
        collection::vec("XD".boxed(), 5).prop_map(|urls| {
            let size = 256; // TODO
            let sha256 = [0; 32]; // TODO
            Installer { urls, size, sha256 }
        })
    }

    /// Generate an empty installer.
    fn empty_installer() -> impl Strategy<Value = Installer> {
        Just(Installer {
            urls: vec![],
            size: 0,
            sha256: [0; 32],
        })
    }

    /// Generate a random [Architecture] uniformly.
    fn arb_architecture() -> impl Strategy<Value = Architecture> {
        prop_oneof![Just(Architecture::X86), Just(Architecture::Arm64)]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::str::FromStr;

    use insta::assert_yaml_snapshot;
    use proptest::prelude::*;

    const TEST_RESPONSE: &[u8] = include_bytes!("../../test-version-response.json");

    proptest! {
        #[test]
        fn test_allow_empty_installers(params in arbitrary::arb_version_parameters(true)) {
            let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE).unwrap();
            let info = VersionInfo::try_from_response(&params, response.signed);
            prop_assert!(info.is_ok());
        }
    }

    // These tests rely on `insta` for snapshot testing. If they fail due to snapshot assertions,
    // then most likely the snapshots need to be updated. The most convenient way to review
    // changes to, and update, snapshots is by running `cargo insta review`.

    /// Test version info response handler (rollout 1, x86)
    #[test]
    fn test_version_info_parser_x86() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_insecure(TEST_RESPONSE)?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: rollout::FULLY_ROLLED_OUT,
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
            rollout: rollout::SUPPORTED_VERSION,
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
            rollout: rollout::SUPPORTED_VERSION,
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
            rollout: rollout::SUPPORTED_VERSION,
            allow_empty: true,
            lowest_metadata_version: 0,
        };

        let info = VersionInfo::try_from_response(&params, response.signed.clone())?;

        // Expect: The available latest versions for x86, where the rollout is non-zero.
        assert_yaml_snapshot!(info);

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: rollout::IGNORE,
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

        assert!(is_version_supported(
            supported_version,
            &response.signed.releases
        ));
        assert!(is_version_supported(
            supported_rollout_zero_version,
            &response.signed.releases
        ));
        assert!(!is_version_supported(
            non_supported_version,
            &response.signed.releases
        ));

        Ok(())
    }
}
