//! Types for handling per-platform metadata

use anyhow::{anyhow, bail, Context};
use mullvad_update::{
    api::HttpVersionInfoProvider,
    format::{self, key},
};
use std::{
    cmp::Ordering,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};
use tokio::{fs, io};

use crate::{
    artifacts,
    io_util::{create_dir_and_write, wait_for_confirm},
};

/// Base URL for metadata found with `meta pull`.
/// Actual JSON files should be stored at `<base url>/<platform>.json`.
const META_REPOSITORY_URL: &str = "https://releases.stagemole.eu/desktop/metadata/";

/// TLS certificate to pin to for `meta pull`.
static PINNED_CERTIFICATE: LazyLock<reqwest::Certificate> = LazyLock::new(|| {
    const CERT_BYTES: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");
    reqwest::Certificate::from_pem(CERT_BYTES).expect("invalid cert")
});

#[derive(Clone, Copy)]
pub enum Platform {
    Windows,
    Linux,
    Macos,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Windows => f.write_str("Windows"),
            Platform::Linux => f.write_str("Linux"),
            Platform::Macos => f.write_str("macOS"),
        }
    }
}

impl FromStr for Platform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "windows" => Ok(Platform::Windows),
            "linux" => Ok(Platform::Linux),
            "macos" => Ok(Platform::Macos),
            other => Err(anyhow!("Invalid platform: {other}")),
        }
    }
}

/// Artifacts paths
pub struct Artifacts {
    pub x86_artifacts: Vec<PathBuf>,
    pub arm64_artifacts: Vec<PathBuf>,
}

impl Platform {
    /// Return array of all platforms
    pub fn all() -> [Self; 3] {
        [Platform::Windows, Platform::Linux, Platform::Macos]
    }

    /// Path to WIP file in `work/` for this platform
    pub fn work_path(&self) -> PathBuf {
        Path::new("work").join(self.local_filename())
    }

    /// Path to signed file in `signed/` for this platform
    pub fn signed_path(&self) -> PathBuf {
        Path::new("signed").join(self.local_filename())
    }

    /// URL that stores the latest published metadata
    pub fn published_url(&self) -> String {
        format!("{META_REPOSITORY_URL}/{}", self.published_filename())
    }

    /// Expected artifacts in `artifacts/` directory
    pub fn artifact_filenames(&self, version: &mullvad_version::Version) -> Artifacts {
        let artifacts_dir = Path::new("artifacts");
        match self {
            Platform::Windows => Artifacts {
                x86_artifacts: vec![artifacts_dir.join(format!("MullvadVPN-{version}_x64.exe"))],
                arm64_artifacts: vec![artifacts_dir.join(format!("MullvadVPN-{version}_arm64.exe"))],
            },
            Platform::Linux => Artifacts {
                x86_artifacts: vec![],
                arm64_artifacts: vec![],
            },
            Platform::Macos => Artifacts {
                x86_artifacts: vec![artifacts_dir.join(format!("MullvadVPN-{version}.pkg"))],
                arm64_artifacts: vec![artifacts_dir.join(format!("MullvadVPN-{version}.pkg"))],
            },
        }
    }

    fn published_filename(&self) -> &str {
        match self {
            Platform::Windows => "windows.json",
            Platform::Linux => "linux.json",
            Platform::Macos => "macos.json",
        }
    }

    fn local_filename(&self) -> &str {
        match self {
            Platform::Windows => "windows.json",
            Platform::Linux => "linux.json",
            Platform::Macos => "macos.json",
        }
    }

    /// Pull latest metadata from repository and store it in `signed/`
    pub async fn pull(&self, assume_yes: bool) -> anyhow::Result<()> {
        let url = self.published_url();

        println!("Pulling {self} metadata from {url}...");

        let version_provider = HttpVersionInfoProvider {
            pinned_certificate: Some(PINNED_CERTIFICATE.clone()),
            url,
            verifying_keys: mullvad_update::keys::TRUSTED_METADATA_SIGNING_PUBKEYS.clone(),
        };
        let response = version_provider
            .get_versions(crate::MIN_VERIFY_METADATA_VERSION)
            .await
            .context("Failed to retrieve versions")?;

        let json = serde_json::to_string_pretty(&response)
            .context("Failed to serialize updated metadata")?;

        let signed_path = self.signed_path();

        // Require confirmation if a signed file exists
        if !assume_yes && signed_path.exists() {
            let msg = format!(
                "This will replace the existing file at {}. Continue?",
                signed_path.display()
            );
            if !wait_for_confirm(&msg).await {
                bail!("Aborted signing");
            }
        }

        println!("Writing metadata to {}", signed_path.display());

        create_dir_and_write(&signed_path, &json).await?;

        println!("Updated {}", signed_path.display());
        Ok(())
    }

    /// Sign version metadata for `platform`.
    /// This will replace the file at `self.signed_path()` with a signed version of
    /// `self.work_path()`.
    pub async fn sign(
        &self,
        secret: key::SecretKey,
        expires_months: usize,
        assume_yes: bool,
    ) -> anyhow::Result<()> {
        let work_path = self.work_path();
        let signed_path = self.signed_path();

        println!(
            "Signing {} and writing it to {}...",
            work_path.display(),
            signed_path.display()
        );

        // Confirm if file exists
        if !assume_yes && signed_path.exists() {
            let msg = format!(
                "This will replace the existing file at {}. Continue?",
                signed_path.display()
            );
            if !wait_for_confirm(&msg).await {
                bail!("Aborted signing");
            }
        }

        // Read unsigned JSON data
        let data = fs::read(work_path).await?;
        let mut response = format::SignedResponse::deserialize_and_verify_insecure(&data)?;

        // Update the expiration date
        response.signed.metadata_expiry = chrono::Utc::now()
            .checked_add_months(chrono::Months::new(
                expires_months.try_into().context("Invalid months")?,
            ))
            .context("Invalid expiry")?;

        println!(
            "Setting metadata expiry to {}",
            response.signed.metadata_expiry
        );

        // Increment metadata version
        let new_version = response.signed.metadata_version + 1;

        println!("Incrementing metadata version to {new_version}");

        // Sign it
        let signed_response = format::SignedResponse::sign(secret, response.signed)?;

        // Update signed data
        let signed_bytes = serde_json::to_string_pretty(&signed_response)
            .context("Failed to serialize signed version")?;
        create_dir_and_write(&signed_path, signed_bytes)
            .await
            .context("Failed to write signed data")?;
        println!("Wrote signed response to {}", signed_path.display());

        Ok(())
    }

    /// Verify the integrity of the platform in `signed/`
    pub async fn verify(&self) -> anyhow::Result<()> {
        let signed_path = self.signed_path();
        println!("Verifying signature of {}...", signed_path.display());
        let bytes = fs::read(signed_path).await.context("Failed to read file")?;

        format::SignedResponse::deserialize_and_verify(
            &mullvad_update::keys::TRUSTED_METADATA_SIGNING_PUBKEYS,
            &bytes,
            crate::MIN_VERIFY_METADATA_VERSION,
        )
        .context("Failed to verify metadata for {platform}: {error}")?;

        Ok(())
    }

    /// Add release to platform in `work/`
    pub async fn add_release(
        &self,
        version: &mullvad_version::Version,
        changes: &str,
        base_urls: &[String],
        rollout: f32,
    ) -> anyhow::Result<()> {
        let installers = self.installers(version, base_urls).await?;

        // Fetch WIP versions and verify that release does not exist
        let work_path = self.work_path();
        println!("Adding {version} from {}", work_path.display());

        let mut work_response = self.read_work().await?;
        if work_response
            .signed
            .releases
            .iter()
            .any(|release| &release.version == version)
        {
            // If it doesn't exist, treat as success
            bail!("Version {version} already exists");
        }

        // Make release
        let new_release = format::Release {
            changelog: changes.to_owned(),
            version: version.clone(),
            installers,
            rollout,
        };

        print_release_info(&new_release);

        work_response.signed.releases.push(new_release);

        let json = serde_json::to_string_pretty(&work_response)
            .context("Failed to serialize updated metadata")?;
        create_dir_and_write(&work_path, &json).await?;

        println!("Added {version} to {}", work_path.display());

        Ok(())
    }

    /// Obtain artifacts checksums and lengths for a given version of this platform in `artifacts/`
    async fn installers(
        &self,
        version: &mullvad_version::Version,
        base_urls: &[String],
    ) -> anyhow::Result<Vec<format::Installer>> {
        let mut installers = vec![];
        let artifacts = self.artifact_filenames(version);
        for artifact in artifacts.arm64_artifacts {
            installers.push(
                artifacts::generate_installer_details(
                    format::Architecture::Arm64,
                    version,
                    base_urls,
                    &artifact,
                )
                .await?,
            );
        }
        for artifact in artifacts.x86_artifacts {
            installers.push(
                artifacts::generate_installer_details(
                    format::Architecture::X86,
                    version,
                    base_urls,
                    &artifact,
                )
                .await?,
            );
        }
        Ok(installers)
    }

    /// List releases for platforms in `work/`
    pub async fn list_releases(&self) -> anyhow::Result<()> {
        let work_path = self.work_path();
        println!("Releases for file {}", work_path.display());

        let mut response = self.read_work().await?;

        if response.signed.releases.is_empty() {
            println!("No releases");
            return Ok(());
        }

        response
            .signed
            .releases
            .sort_by(|a, b| b.version.partial_cmp(&a.version).unwrap_or(Ordering::Equal));

        for release in response.signed.releases {
            print_release_info(&release);
        }
        Ok(())
    }

    /// Remove version/release in `work/`
    pub async fn remove_release(&self, version: &mullvad_version::Version) -> anyhow::Result<()> {
        let work_path = self.work_path();
        println!("Removing {version} from {}", work_path.display());

        let mut work_response = self.read_work().await?;

        let Some(found_release_ind) = work_response
            .signed
            .releases
            .iter()
            .position(|release| &release.version == version)
        else {
            // If it doesn't exist, treat as success
            return Ok(());
        };

        let removed_release = work_response.signed.releases.swap_remove(found_release_ind);

        print_release_info(&removed_release);

        let json = serde_json::to_string_pretty(&work_response)
            .context("Failed to serialize updated metadata")?;
        create_dir_and_write(&work_path, &json).await?;

        println!("Removed {version} in {}", work_path.display());

        Ok(())
    }

    /// Modify version/release in `work/`
    pub async fn modify_release(
        &self,
        version: &mullvad_version::Version,
        rollout: Option<f32>,
    ) -> anyhow::Result<()> {
        let work_path = self.work_path();
        println!("Modifying {version} in {}", work_path.display());

        let mut work_response = self.read_work().await?;

        let Some(release) = work_response
            .signed
            .releases
            .iter_mut()
            .find(|release| &release.version == version)
        else {
            bail!("{version} not found in {}", work_path.display());
        };

        if let Some(new_rollout) = rollout {
            release.rollout = new_rollout;
        }

        print_release_info(release);

        let json = serde_json::to_string_pretty(&work_response)
            .context("Failed to serialize updated metadata")?;
        create_dir_and_write(&work_path, &json).await?;

        println!("Updated {version} in {}", work_path.display());

        Ok(())
    }

    /// Reads the metadata for `platform` in the work directory.
    /// If the file doesn't exist, this returns a new, empty response.
    async fn read_work(&self) -> anyhow::Result<format::SignedResponse> {
        let work_path = self.work_path();
        let bytes = match fs::read(&work_path).await {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // Return empty response
                return Ok(format::SignedResponse {
                    signatures: vec![],
                    signed: format::Response::default(),
                });
            }
            Err(error) => bail!("Failed to read {}: {error}", work_path.display()),
        };
        // Note: We don't need to verify the signature here
        format::SignedResponse::deserialize_and_verify_insecure(&bytes)
    }
}

/// Print release info:
/// Version: 2025.3 (arm, x86) (50%)
/// <Changelog>
fn print_release_info(release: &format::Release) {
    let mut architectures: Vec<_> = release
        .installers
        .iter()
        .map(|installer| installer.architecture.to_string())
        .collect();
    architectures.dedup();
    let architectures = architectures.join(", ");

    println!(
        "- {} ({}) ({}%)",
        release.version,
        architectures,
        (release.rollout * 100.) as u32
    );
}
