use std::{fmt, io, path::Path, process::Stdio, time::Duration};

use anyhow::{anyhow, bail, Context};
use notify::{RecursiveMode, Watcher};
use std::io::Write;
use tokio::{fs::File, process::Command};

use crate::device::AccountManagerHandle;

/// Bump filehandle limit
pub fn bump_filehandle_limit() {
    let mut limits = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `&mut limits` is a valid pointer parameter for the getrlimit syscall
    let status = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) };
    if status != 0 {
        log::error!(
            "Failed to get file handle limits: {}-{}",
            io::Error::from_raw_os_error(status),
            status
        );
        return;
    }

    const INCREASED_FILEHANDLE_LIMIT: u64 = 1024;
    // if file handle limit is already big enough, there's no reason to decrease it.
    if limits.rlim_cur >= INCREASED_FILEHANDLE_LIMIT {
        return;
    }

    limits.rlim_cur = INCREASED_FILEHANDLE_LIMIT;
    // SAFETY: `&limits` is a valid pointer parameter for the getrlimit syscall
    let status = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limits) };
    if status != 0 {
        log::error!(
            "Failed to set file handle limit to {}: {}-{}",
            INCREASED_FILEHANDLE_LIMIT,
            io::Error::from_raw_os_error(status),
            status
        );
    }
}

/// Detect when the app bundle is deleted
pub async fn handle_app_bundle_removal(
    account_manager_handle: AccountManagerHandle,
) -> anyhow::Result<()> {
    const UNINSTALL_SCRIPT: &[u8] = include_bytes!("../../dist-assets/uninstall_macos.sh");
    const UNINSTALL_SCRIPT_PATH: &str = "/var/root/uninstall_mullvad.sh";

    let daemon_path = std::env::current_exe().context("Failed to get daemon path")?;

    let (fs_notify_tx, mut fs_notify_rx) = tokio::sync::mpsc::channel(1);
    let daemon_path_2 = daemon_path.clone();
    let mut fs_watcher =
        notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            _ = event;
            if !daemon_path_2.exists() {
                _ = fs_notify_tx.try_send(());
            }
        })
        .context("Failed to start filesystem watcher")?;

    fs_watcher
        .watch(&daemon_path, RecursiveMode::NonRecursive)
        .context(anyhow!("Failed to watch {daemon_path:?}"))?;

    let _file_was_deleted = fs_notify_rx
        .recv()
        .await
        .context("Filesystem watcher stopped unexpectedly")?;
    drop(fs_watcher);

    // Create file to log output from uninstallation process.
    // This is useful since the daemon will be killed during uninstallation.
    let mut log_file = async {
        let log_path: std::path::PathBuf = mullvad_paths::log_dir()?.join("uninstall.log");

        let file = File::create(log_path).await?;
        anyhow::Ok(file.into_std().await)
    }
    .await
    .inspect_err(|e| {
        log::warn!("Failed to create uninstaller log-file: {e:#?}");
    })
    .ok();

    // Log to both daemon log and uninstaller log.
    let mut log = |msg: fmt::Arguments<'_>| {
        log::info!("{msg}");
        if let Some(log_file) = &mut log_file {
            let _ = writeln!(log_file, "{msg}");
        }
    };

    log(format_args!(
        "{daemon_path:?} was removed. Running uninstaller."
    ));

    tokio::fs::write(UNINSTALL_SCRIPT_PATH, UNINSTALL_SCRIPT).await?;

    // If reset_firewall errors, log the error and continue anyway.
    log(format_args!("Resetting firewall"));
    if let Err(error) = reset_firewall() {
        log(format_args!("{error:#?}"));
    }

    // Remove the current device from the account
    log(format_args!("Logging out"));
    if let Err(error) = account_manager_handle.logout().await {
        log(format_args!("Failed to remove device: {error:#?}"));
    }

    // This will kill the daemon.
    log(format_args!("Running {UNINSTALL_SCRIPT_PATH:?}"));
    let mut cmd = Command::new("/bin/bash");
    cmd
        .arg(UNINSTALL_SCRIPT_PATH)
        // Don't prompt for confirmation.
        .arg("--yes") 
        // Spawn as its own process group.
        // This prevents the command from being killed when the daemon is killed.
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(log_file) = log_file {
        cmd.stdout(log_file.try_clone().context("Failed to clone log fd")?);
        cmd.stderr(log_file);
    };
    cmd.spawn().context("Failed to spawn uninstaller script")?;

    Ok(())
}

fn reset_firewall() -> anyhow::Result<()> {
    talpid_core::firewall::Firewall::new()
        .context("Failed to create firewall instance")?
        .reset_policy()
        .context("Failed to reset firewall policy")
}
