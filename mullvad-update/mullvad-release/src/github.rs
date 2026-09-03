//! Tools for the GitHub repository.

use anyhow::Context;
use mullvad_update::tls::build_client_config;

/// Obtain changes.txt for a given version/tag from the GitHub repository
pub async fn fetch_changes_text(version: &mullvad_version::Version) -> anyhow::Result<String> {
    let github_changes_url = format!(
        "https://raw.githubusercontent.com/mullvad/mullvadvpn-app/refs/tags/{version}/desktop/packages/mullvad-vpn/changes.txt"
    );
    // reqwest has no bundled crypto provider, so supply a preconfigured
    // aws-lc-rs rustls ClientConfig with webpki roots.
    let client = reqwest::Client::builder()
        .use_preconfigured_tls(build_client_config(None, false))
        .build()
        .context("Failed to build HTTP client")?;
    let changes = client
        .get(github_changes_url)
        .send()
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
