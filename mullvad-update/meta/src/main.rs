//! See [Opt].

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use std::{
    cmp::Ordering,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{
    fs,
    io::{self, AsyncSeekExt, BufReader},
};

use mullvad_update::{
    api::HttpVersionInfoProvider,
    format::{self, key, SignedResponse},
    verify::Sha256Verifier,
};

/// Metadata expiry to use when not specified (months from now)
#[allow(dead_code)]
const DEFAULT_EXPIRY_MONTHS: usize = 6;

/// Rollout to use when not specified
const DEFAULT_ROLLOUT: f32 = 1.;

/// Base URL for metadata found with `meta pull`.
/// Actual JSON files should be stored at `<base url>/updates-<platform>.json`.
const META_REPOSITORY_URL: &str = "https://releases.mullvad.net/desktop/metadata/";

/// Lowest version to accept using 'verify'
const MIN_VERIFY_METADATA_VERSION: usize = 0;

/// Verification public key
const VERIFYING_PUBKEY: &str = include_str!("../../test-pubkey");

/// A tool that generates signed Mullvad version metadata.
///
/// Unsigned work is stored in `work/`, and signed work is stored in `signed/`
#[derive(Parser)]
pub enum Opt {
    /// Generate an ed25519 secret key
    GenerateKey,

    /// Create empty metadata files in work directory
    CreateMetadataFile {
        /// Platforms to write template for
        platforms: Vec<Platform>,
    },

    /// Download version metadata from releases.mullvad.net or API endpoint and store it in
    /// `signed/`
    Pull {
        /// Platforms to write template for
        platforms: Vec<Platform>,

        /// Replace signed files without user input
        #[arg(long, short = 'y')]
        assume_yes: bool,
    },

    /// List releases in `work/`
    ListReleases {
        /// Platforms to list releases for. All if none are specified
        platforms: Vec<Platform>,
    },

    /// Add release to `work/`
    AddRelease {
        /// Version to add
        version: mullvad_version::Version,
        /// Platforms to add releases for. All if none are specified
        platforms: Vec<Platform>,
        /// Rollout percentage (default is 1)
        #[arg(long)]
        rollout: Option<f32>,
    },

    /// Remove release from `work/`
    RemoveRelease {
        /// Version to remove
        version: mullvad_version::Version,
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
    },

    /// Modify release in `work/`
    ModifyRelease {
        /// Version to modify
        version: mullvad_version::Version,
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
        /// Rollout percentage. The default is 1
        #[arg(long)]
        rollout: Option<f32>,
    },

    /// Sign using an ed25519 key and output the signed metadata to `signed/`
    Sign {
        /// Secret ed25519 key used for signing, as hexadecimal string
        secret: key::SecretKey,
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
        /// When the metadata expires, in months from now
        #[arg(long, default_value_t = DEFAULT_EXPIRY_MONTHS)]
        expiry: usize,
        /// Replace signed files without user input
        #[arg(long, short = 'y')]
        assume_yes: bool,
    },

    /// Verify that payloads are signed by a given ed25519 pubkey
    Verify {
        /// Platforms to remove releases for. All if none are specified
        platforms: Vec<Platform>,
    },
}

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
struct Artifacts {
    x86_artifacts: Vec<PathBuf>,
    arm64_artifacts: Vec<PathBuf>,
}

impl Platform {
    /// Path to WIP file in `work/` for this platform
    fn work_path(&self) -> PathBuf {
        Path::new("work").join(self.local_filename())
    }

    /// Path to signed file in `signed/` for this platform
    fn signed_path(&self) -> PathBuf {
        Path::new("signed").join(self.local_filename())
    }

    /// URL that stores the latest published metadata
    fn published_url(&self) -> String {
        format!("{META_REPOSITORY_URL}/{}", self.published_filename())
    }

    /// Expected artifacts in `artifacts/` directory
    fn artifact_filenames(&self, version: &mullvad_version::Version) -> Artifacts {
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
            Platform::Windows => "updates-windows.json",
            Platform::Linux => "updates-linux.json",
            Platform::Macos => "updates-macos.json",
        }
    }

    fn local_filename(&self) -> &str {
        match self {
            Platform::Windows => "windows.json",
            Platform::Linux => "linux.json",
            Platform::Macos => "macos.json",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::GenerateKey => {
            println!("{}", key::SecretKey::generate().to_string());
            Ok(())
        }
        Opt::CreateMetadataFile { platforms } => {
            let json = serde_json::to_string_pretty(&SignedResponse {
                signatures: vec![],
                signed: format::Response::default(),
            })
            .expect("Failed to serialize empty response");
            for platform in all_platforms_if_empty(platforms) {
                let work_path = platform.work_path();
                println!("Adding empty template to {}", work_path.display());
                create_dir_and_write(work_path, &json).await?;
            }
            Ok(())
        }
        Opt::Pull {
            platforms,
            assume_yes,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                let url = platform.published_url();

                println!("Pulling {platform} metadata from {url}...");

                // Pull latest metadata
                let verifying_key =
                    key::VerifyingKey::from_hex(VERIFYING_PUBKEY).expect("Invalid pubkey");

                let version_provider = HttpVersionInfoProvider {
                    // TODO: pin
                    pinned_certificate: None,
                    url,
                    verifying_key,
                };
                let response = version_provider
                    .get_versions(MIN_VERIFY_METADATA_VERSION)
                    .await
                    .context("Failed to retrieve versions")?;

                let json = serde_json::to_string_pretty(&response)
                    .context("Failed to serialize updated metadata")?;

                let signed_path = platform.signed_path();

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

                println!("Writing metadata to {}", signed_path.display());

                create_dir_and_write(&signed_path, &json).await?;

                println!("Updated {}", signed_path.display());
            }

            Ok(())
        }
        Opt::Sign {
            secret,
            platforms,
            expiry,
            assume_yes,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                sign(platform, secret.clone(), expiry, assume_yes)
                    .await
                    .context("Failed to sign file")?;
            }
            Ok(())
        }
        Opt::ListReleases { platforms } => {
            for platform in all_platforms_if_empty(platforms) {
                list_releases(platform).await?;
                println!();
            }
            Ok(())
        }
        Opt::AddRelease {
            version,
            platforms,
            rollout,
        } => {
            // Obtain changes.txt from GitHub
            let changes = get_github_tag_changes(&version).await?;
            println!("\nchanges.txt for tag {version}:\n\n-- begin\n{changes}\n--end\n\n");

            for platform in all_platforms_if_empty(platforms) {
                // Obtain artifacts checksums and lengths
                let mut installers = vec![];
                let artifacts = platform.artifact_filenames(&version);
                for artifact in artifacts.arm64_artifacts {
                    installers.push(
                        generate_installer_details_for_artifact(
                            format::Architecture::Arm64,
                            &artifact,
                        )
                        .await?,
                    );
                }
                for artifact in artifacts.x86_artifacts {
                    installers.push(
                        generate_installer_details_for_artifact(
                            format::Architecture::X86,
                            &artifact,
                        )
                        .await?,
                    );
                }

                add_release(&version, platform, &changes, installers, rollout).await?;
            }
            Ok(())
        }
        Opt::RemoveRelease { version, platforms } => {
            for platform in all_platforms_if_empty(platforms) {
                remove_release(&version, platform).await?;
            }
            Ok(())
        }
        Opt::ModifyRelease {
            version,
            platforms,
            rollout,
        } => {
            for platform in all_platforms_if_empty(platforms) {
                modify_release(&version, platform, rollout).await?;
            }
            Ok(())
        }
        Opt::Verify { platforms } => {
            let mut any_failed = false;
            for platform in all_platforms_if_empty(platforms) {
                let signed_path = platform.signed_path();
                let bytes = fs::read(signed_path).await.context("Failed to read file")?;

                // TODO: Actual key
                let public_key = key::VerifyingKey::from_hex(include_str!("../../test-pubkey"))
                    .expect("Invalid pubkey");

                if let Err(error) = format::SignedResponse::deserialize_and_verify(
                    &public_key,
                    &bytes,
                    MIN_VERIFY_METADATA_VERSION,
                ) {
                    any_failed |= true;
                    eprintln!("Failed to verify metadata for {platform}: {error}");
                }
            }
            if any_failed {
                bail!("Some signatures failed to be verified");
            }
            Ok(())
        }
    }
}

async fn generate_installer_details_for_artifact(
    architecture: format::Architecture,
    artifact: &Path,
) -> anyhow::Result<format::Installer> {
    let mut file = fs::File::open(artifact)
        .await
        .context(format!("Failed to open file at {}", artifact.display()))?;
    file.seek(std::io::SeekFrom::End(0))
        .await
        .context("Failed to seek to end")?;
    let file_size = file
        .stream_position()
        .await
        .context("Failed to get file size")?;
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .context("Failed to reset file pos")?;
    let file = BufReader::new(file);

    let checksum = Sha256Verifier::generate_hash(file)
        .await
        .context("Failed to compute checksum")?;

    Ok(format::Installer {
        architecture,
        // TODO: fetch cdns from config
        urls: vec![],
        size: file_size.try_into().context("Invalid file size")?,
        sha256: hex::encode(checksum),
    })
}

async fn list_releases(platform: Platform) -> anyhow::Result<()> {
    let work_path = platform.work_path();
    println!("Releases for file {}", work_path.display());

    let mut response = read_work(platform).await?;

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

async fn add_release(
    version: &mullvad_version::Version,
    platform: Platform,
    changes: &str,
    installers: Vec<format::Installer>,
    rollout: Option<f32>,
) -> anyhow::Result<()> {
    // Fetch WIP versions and verify that release does not exist
    let work_path = platform.work_path();
    println!("Adding {version} from {}", work_path.display());

    let mut response = read_work(platform).await?;
    if response
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
        installers: installers.to_owned(),
        rollout: rollout.unwrap_or(DEFAULT_ROLLOUT),
    };

    print_release_info(&new_release);

    response.signed.releases.push(new_release);

    let json =
        serde_json::to_string_pretty(&response).context("Failed to serialize updated metadata")?;
    create_dir_and_write(&work_path, &json).await?;

    println!("Added {version} to {}", work_path.display());

    Ok(())
}

// Obtain changes.txt for a given version from the GitHub repository
async fn get_github_tag_changes(version: &mullvad_version::Version) -> anyhow::Result<String> {
    let github_changes_url = format!("https://raw.githubusercontent.com/mullvad/mullvadvpn-app/refs/tags/{version}/desktop/packages/mullvad-vpn/changes.txt");
    let changes = reqwest::get(github_changes_url)
        .await
        .context("Failed to retrieve changes.txt (tag missing?)")?;
    if let Err(err) = changes.error_for_status_ref() {
        return Err(err).context("Error status returned when downloading changes.txt");
    }
    changes
        .text()
        .await
        .context("Failed to retrieve text for changes.txt (tag missing?)")
}

async fn remove_release(
    version: &mullvad_version::Version,
    platform: Platform,
) -> anyhow::Result<()> {
    let work_path = platform.work_path();
    println!("Removing {version} from {}", work_path.display());

    let mut response = read_work(platform).await?;

    let Some(found_release_ind) = response
        .signed
        .releases
        .iter()
        .position(|release| &release.version == version)
    else {
        // If it doesn't exist, treat as success
        return Ok(());
    };

    let removed_release = response.signed.releases.swap_remove(found_release_ind);

    print_release_info(&removed_release);

    let json =
        serde_json::to_string_pretty(&response).context("Failed to serialize updated metadata")?;
    create_dir_and_write(&work_path, &json).await?;

    println!("Removed {version} in {}", work_path.display());

    Ok(())
}

async fn modify_release(
    version: &mullvad_version::Version,
    platform: Platform,
    rollout: Option<f32>,
) -> anyhow::Result<()> {
    let work_path = platform.work_path();
    println!("Modifying {version} in {}", work_path.display());

    let mut response = read_work(platform).await?;

    let Some(release) = response
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

    print_release_info(&release);

    let json =
        serde_json::to_string_pretty(&response).context("Failed to serialize updated metadata")?;
    create_dir_and_write(&work_path, &json).await?;

    println!("Updated {version} in {}", work_path.display());

    Ok(())
}

/// Reads the metadata for `platform` in the work directory.
/// If the file doesn't exist, this returns a new, empty response.
async fn read_work(platform: Platform) -> anyhow::Result<format::SignedResponse> {
    let work_path = platform.work_path();
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

fn all_platforms_if_empty(platforms: Vec<Platform>) -> Vec<Platform> {
    if platforms.is_empty() {
        return vec![Platform::Windows, Platform::Linux, Platform::Macos];
    }
    platforms
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

/// Sign version metadata for `platform`.
/// This will replace the file at `platform.signed_path()` with a signed version of
/// `platform.work_path()`.
async fn sign(
    platform: Platform,
    secret: key::SecretKey,
    expires_months: usize,
    assume_yes: bool,
) -> anyhow::Result<()> {
    let work_path = platform.work_path();
    let signed_path = platform.signed_path();

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

/// Recursively create directories and write to 'file'
async fn create_dir_and_write(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> anyhow::Result<()> {
    let path = path.as_ref();

    let parent_dir = path.parent().context("Missing parent directory")?;
    fs::create_dir_all(parent_dir)
        .await
        .context("Failed to create directories")?;

    fs::write(path, contents).await?;
    Ok(())
}

/// Wait for user to respond with yes or no
/// This returns `false` if reading from stdin fails
async fn wait_for_confirm(prompt: &str) -> bool {
    const DEFAULT: bool = true;

    print!("{prompt}");
    if DEFAULT {
        println!(" [Y/n]");
    } else {
        println!(" [y/N]");
    }

    tokio::task::spawn_blocking(|| {
        let mut s = String::new();
        let stdin = std::io::stdin();

        loop {
            stdin.read_line(&mut s).context("Failed to read line")?;

            match s.trim().to_ascii_lowercase().as_str() {
                "" => break Ok::<bool, anyhow::Error>(DEFAULT),
                "y" | "ye" | "yes" => break Ok(true),
                "n" | "no" => break Ok(false),
                _ => (),
            }
        }
    })
    .await
    .unwrap()
    .unwrap_or(false)
}
