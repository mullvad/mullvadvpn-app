use std::time::Duration;
#[cfg(target_os = "macos")]
use std::{path::Path, process::Stdio};

use anyhow::{Context, bail};
use futures::StreamExt;
use indicatif::ProgressStyle;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::version::AppUpgradeEvent;
#[cfg(target_os = "macos")]
use tokio::process::Command;

const SPIN_INTERVAL: Duration = Duration::from_millis(50);

pub async fn update() -> anyhow::Result<()> {
    // TODO: show the time estimate?
    // TODO: ctrl-c -> abort

    let mut rpc = MullvadProxyClient::new()
        .await
        .context("Failed to connect to mullvad-daemon")?;

    let info = rpc
        .get_version_info()
        .await
        .context("Failed to get version info")?;

    let Some(update_version) = info.suggested_upgrade else {
        println!("No update available.");
        return Ok(());
    };

    if let Some(_installer_path) = update_version.verified_installer_path {
        // TODO:
        //todo!("just install here")
        rpc.app_upgrade_abort().await.unwrap();
    };

    let cache_dir = rpc.get_app_upgrade_cache_dir().await.unwrap();

    println!("Update available: {}", update_version.version);
    println!("Target directory: {cache_dir}");
    if !update_version.changelog.trim().is_empty() {
        println!("Changes:");
        for change in update_version.changelog.lines() {
            println!("\t{change}");
        }
    }
    println!("---");

    let mut update_events = rpc.app_upgrade_events_listen().await?;

    // FIXME
    rpc.app_upgrade_abort().await.unwrap();

    rpc.app_upgrade()
        .await
        .context("Failed to initiate update")?;

    let mut download_progress = None;

    while let Some(evt) = update_events.next().await {
        match evt.context("Update event error")? {
            AppUpgradeEvent::DownloadStarting => {
                //println!("Downloading {}...", update_version.);
            }
            AppUpgradeEvent::DownloadProgress(progress) => {
                // TODO: Estimated time left

                let bar = download_progress.get_or_insert_with(|| {
                    indicatif::ProgressBar::new(100)
                        .with_message(format!(
                            "Downloading version {} from {}",
                            update_version.version, progress.server
                        ))
                        .with_style(
                            ProgressStyle::with_template(
                                "  {msg}:\n  {bar:40} {percent:>4}% | {elapsed_precise}",
                            )
                            .unwrap()
                            .progress_chars("##-"),
                        )
                });
                bar.set_position(progress.progress.into());
            }
            AppUpgradeEvent::VerifyingInstaller => {
                if let Some(bar) = download_progress.take() {
                    bar.finish_with_message("Download complete");
                }
                let bar = indicatif::ProgressBar::new_spinner().with_message("Verifying installer");
                bar.enable_steady_tick(SPIN_INTERVAL);
                download_progress = Some(bar);
            }
            AppUpgradeEvent::VerifiedInstaller => {
                let path = rpc
                    .get_version_info()
                    .await
                    .context("Failed to get version info")?
                    .suggested_upgrade
                    .context("Missing update info")?
                    .verified_installer_path
                    .context("Missing installer path")?;

                if let Some(bar) = download_progress.take() {
                    bar.finish_with_message(format!("Installer verified: {}", path.display()));
                }

                let bar = indicatif::ProgressBar::new_spinner().with_message("Installing");
                bar.enable_steady_tick(SPIN_INTERVAL);
                install_package(path).await?;

                bar.finish_with_message("Update complete");

                break;
            }
            AppUpgradeEvent::Aborted => {
                println!("Update aborted.");
                break;
            }
            AppUpgradeEvent::Error(err) => {
                // TODO: improved err message
                eprintln!("Update failed: {err:?}");
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn install_package(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args(["/usr/sbin/installer", "-pkg"]);
    cmd.arg(path.as_ref());
    cmd.args(["-target", "/"]);
    cmd.kill_on_drop(true);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let status = cmd
        .spawn()
        .context("Failed to spawn installer")?
        .wait()
        .await
        .context("installer failed")?;
    if !status.success() {
        let code = status.code().context("No error code")?;
        bail!("Installer failed: {code}")
    }
    Ok(())
}
