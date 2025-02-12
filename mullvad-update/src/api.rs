//! Fetch information about app versions from the Mullvad API

use anyhow::Context;

use crate::format;

/// Parameters for [VersionInfoProvider]
#[derive(Debug)]
pub struct VersionParameters {
    /// Architecture to retrieve data for
    pub architecture: VersionArchitecture,
    /// Rollout threshold. Any version in the response below this threshold will be ignored
    pub rollout: f32,
}

/// Installer architecture
pub type VersionArchitecture = format::Architecture;

/// See [module-level](self) docs.
#[async_trait::async_trait]
pub trait VersionInfoProvider {
    /// Return info about the stable version
    async fn get_version_info(params: VersionParameters) -> anyhow::Result<VersionInfo>;
}

/// Contains information about all versions
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Stable version info
    pub stable: Version,
    /// Beta version info (if available and newer than `stable`).
    /// If latest stable version is newer, this will be `None`.
    pub beta: Option<Version>,
}

/// Contains information about a version for the current target
#[derive(Debug, Clone)]
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

/// Obtain version data from the Mullvad API
pub struct ApiVersionInfoProvider;

#[async_trait::async_trait]
impl VersionInfoProvider for ApiVersionInfoProvider {
    async fn get_version_info(params: VersionParameters) -> anyhow::Result<VersionInfo> {
        // FIXME: Replace with actual API response
        use format::*;

        const TEST_PUBKEY: &str =
            "4d35f5376f1f58c41b2a0ee4600ae7811eace354f100227e853994deef38942d";
        let pubkey = hex::decode(TEST_PUBKEY).unwrap();
        let verifying_key =
            ed25519_dalek::VerifyingKey::from_bytes(&pubkey.try_into().unwrap()).unwrap();

        let response = SignedResponse::deserialize_and_verify(
            format::key::VerifyingKey(verifying_key),
            include_bytes!("../test-version-response.json"),
        )?;

        VersionInfo::try_from_response(&params, response.signed)
    }
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
    fn try_from_response(
        params: &VersionParameters,
        response: format::Response,
    ) -> anyhow::Result<Self> {
        let mut releases: Vec<_> = response
            .releases
            .into_iter()
            // Filter out releases that are not rolled out to us
            .filter(|release| release.rollout >= params.rollout)
            // Include only installers for the requested architecture
            .flat_map(|release| {
                release
                    .installers
                    .into_iter()
                    .filter(|installer| params.architecture == installer.architecture)
                    // Map each artifact to a [IntermediateVersion]
                    .map(move |installer| {
                        IntermediateVersion {
                            version: release.version.clone(),
                            changelog: release.changelog.clone(),
                            installer,
                        }
                    })
            })
            .collect();

        // Sort releases by version
        releases.sort_by(|a, b| mullvad_version::Version::version_ordering(&a.version, &b.version));

        // Fail if there are duplicate versions
        // Important! This must occur after sorting
        if let Some(dup_version) = Self::find_duplicate_version(&releases) {
            anyhow::bail!("API response contains at least one duplicated version: {dup_version}");
        }

        // Find latest stable version
        let stable = releases
            .iter()
            .rfind(|release| release.version.is_stable() && !release.version.is_dev());
        let Some(stable) = stable.cloned() else {
            anyhow::bail!("No stable version found");
        };

        // Find the latest beta version
        let beta = releases
            .iter()
            // Find most recent beta version
            .rfind(|release| release.version.beta().is_some() && !release.version.is_dev())
            // If the latest beta version is older than latest stable, dispose of it
            .filter(|release| release.version.version_ordering(&stable.version).is_gt())
            .cloned();

        Ok(Self {
            stable: Version::try_from(stable)?,
            beta: beta.map(|beta| Version::try_from(beta)).transpose()?,
        })
    }

    /// Returns the first duplicated version found in `releases`.
    /// `None` is returned if there are no duplicates.
    /// NOTE: `releases` MUST be sorted
    fn find_duplicate_version(
        releases: &[IntermediateVersion],
    ) -> Option<&mullvad_version::Version> {
        releases
            .windows(2)
            .find(|pair| {
                mullvad_version::Version::version_ordering(&pair[0].version, &pair[1].version)
                    .is_eq()
            })
            .map(|pair| &pair[0].version)
    }
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
    use super::*;

    /// Test API version responses can be parsed
    #[test]
    fn test_api_version_info_provider_parser() -> anyhow::Result<()> {
        let response = format::SignedResponse::deserialize_and_verify_insecure(include_bytes!(
            "../test-version-response.json"
        ))?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: 1.,
        };

        VersionInfo::try_from_signed_response(&params, response)?;

        Ok(())
    }
}
