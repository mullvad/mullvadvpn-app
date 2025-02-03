//! Fetch information about app versions from the Mullvad API

/// See [module-level](self) docs.
#[async_trait::async_trait]
pub trait VersionInfoProvider {
    /// Return info about the stable version
    async fn get_version_info() -> anyhow::Result<VersionInfo>;
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
    pub version: String,
    /// URLs to use for downloading the app installer
    pub urls: Vec<String>,
    /// Size of installer, in bytes
    pub size: usize,
    /// URLs pointing to app PGP signatures
    pub signature_urls: Vec<String>,
}

/// Use hardcoded URL to fetch installer
/// TODO: This is temporary
pub struct LatestVersionInfoProvider;

#[async_trait::async_trait]
impl VersionInfoProvider for LatestVersionInfoProvider {
    async fn get_version_info() -> anyhow::Result<VersionInfo> {
        Ok(VersionInfo {
            stable: Version {
                version: "2025.3".to_string(),
                urls: vec!["https://mullvad.net/en/download/app/exe/latest".to_owned()],
                size: 200 * 1024 * 1024,
                signature_urls: vec![
                    "https://mullvad.net/en/download/app/exe/latest/signature".to_owned()
                ],
            },
            beta: Some(Version {
                version: "2025.3-beta1".to_string(),
                urls: vec!["https://mullvad.net/en/download/app/exe/latest-beta".to_owned()],
                size: 200 * 1024 * 1024,
                signature_urls: vec![
                    "https://mullvad.net/en/download/app/exe/latest-beta/signature".to_owned(),
                ],
            }),
        })
    }
}
