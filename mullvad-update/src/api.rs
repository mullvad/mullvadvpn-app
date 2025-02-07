//! Fetch information about app versions from the Mullvad API

use anyhow::Context;

use crate::deserializer;

/// Parameters for [VersionInfoProvider]
#[derive(Debug)]
pub struct VersionParameters {
    /// Architecture to retrieve data for
    pub architecture: VersionArchitecture,
    /// Rollout threshold. Any version in the response below this threshold will be ignored
    pub rollout: f32,
}

/// Architecture to retrieve data for
#[derive(Debug, Clone, Copy)]
pub enum VersionArchitecture {
    /// x86-64 architecture
    X86,
    /// ARM64 architecture
    Arm64,
}

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
    /// Beta version info
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
        use deserializer::*;

        const TEST_PUBKEY: &str =
            "AEC24A08466F3D6A1EDCDB2AD3C234428AB9D991B6BEA7F53CB9F172E6CB40D8";
        let pubkey = hex::decode(TEST_PUBKEY).unwrap();
        let verifying_key =
            ed25519_dalek::VerifyingKey::from_bytes(&pubkey.try_into().unwrap()).unwrap();

        let response = SignedResponse::deserialize_and_verify(
            VerifyingKey(verifying_key),
            include_bytes!("../test-version-response.json"),
        )?;

        VersionInfo::try_from_signed_response(&params, response)
    }
}

impl VersionInfo {
    /// Convert signed response data to public version type
    /// NOTE: `response` is assumed to be verified and untampered. It is not verified.
    fn try_from_signed_response(
        params: &VersionParameters,
        response: deserializer::SignedResponse,
    ) -> anyhow::Result<Self> {
        let stable = Version::try_from_signed_response(params, response.signed.stable)?;
        let beta = response
            .signed
            .beta
            .map(|response| Version::try_from_signed_response(params, response))
            .transpose()
            .context("Failed to parse beta version")?;

        Ok(Self { stable, beta })
    }
}

impl Version {
    /// Convert response data to public version type
    fn try_from_signed_response(
        params: &VersionParameters,
        response: deserializer::VersionResponse,
    ) -> anyhow::Result<Self> {
        // Check if the rollout version is acceptable according to threshold
        if let Some(next) = response.next {
            if next.rollout >= params.rollout {
                // Use the version being rolled out
                return Self::try_for_arch(params, next.version);
            }
        }

        // Return the version not being rolled out
        Self::try_for_arch(params, response.current)
    }

    /// Convert version response to the public version type for a given architecture
    /// This may fail if the current architecture isn't included in the response
    fn try_for_arch(
        params: &VersionParameters,
        response: deserializer::SpecificVersionResponse,
    ) -> anyhow::Result<Self> {
        let installer = match params.architecture {
            VersionArchitecture::X86 => response.installers.x86,
            VersionArchitecture::Arm64 => response.installers.arm64,
        };
        let installer = installer.context("Installer missing for architecture")?;
        let sha256 = hex::decode(installer.sha256)
            .context("Invalid checksum hex")?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid checksum length"))?;

        Ok(Self {
            changelog: response.changelog,
            version: response.version,
            urls: installer.urls,
            size: installer.size,
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
        let response = deserializer::SignedResponse::deserialize_and_verify_insecure(
            include_bytes!("../test-version-response.json"),
        )?;

        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: 1.,
        };

        VersionInfo::try_from_signed_response(&params, response)?;

        Ok(())
    }
}
