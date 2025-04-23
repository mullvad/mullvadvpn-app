use std::{ffi::OsStr, fmt, io, path::Path, process::Stdio};

use anyhow::{anyhow, Context};
use libc::{pid_t, PROX_FDTYPE_VNODE};
use notify::{RecursiveMode, Watcher};
use std::io::Write;
use talpid_macos::process::{
    get_file_desc_vnode_path, list_pids, process_bsdinfo, process_file_descriptors, process_path,
};
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
    /// Uninstall script to run if the .app disappears
    const UNINSTALL_SCRIPT: &[u8] = include_bytes!("../../dist-assets/uninstall_macos.sh");

    /// Path to extract the uninstall script to.
    /// This directory must be owned by root to prevent privilege escalation.
    const UNINSTALL_SCRIPT_PATH: &str = "/var/root/uninstall_mullvad.sh";

    /// Mullvad app install path
    const APP_PATH: &str = "/Applications/Mullvad VPN.app";

    let daemon_path = std::env::current_exe().context("Failed to get daemon path")?;

    // Ignore app removal if the daemon isn't installed in the app directory
    if !daemon_path.starts_with(APP_PATH) {
        log::trace!("Stopping handle_app_bundle_removal as the daemon is not installed");
        return Ok(());
    }

    let (fs_notify_tx, mut fs_notify_rx) = tokio::sync::mpsc::channel(1);
    let mut fs_watcher =
        notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            // Ignore access events
            let is_access_event = event.map(|evt| evt.kind.is_access()).unwrap_or(false);

            // Check if the daemon binary still exists
            if !is_access_event && !daemon_path.exists() {
                _ = fs_notify_tx.try_send(());
            }
        })
        .context("Failed to start filesystem watcher")?;

    fs_watcher
        .watch(Path::new(APP_PATH), RecursiveMode::Recursive)
        .context(anyhow!("Failed to watch {APP_PATH}"))?;

    fs_notify_rx
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

    log(format_args!("{APP_PATH} was removed."));

    // TODO: This check can be removed once we no longer care about downgrades to
    // versions that didn't unload the daemon in preinstall instead of postinstall.
    // E.g., a year after we released version 2025.7
    if mullvad_installer_is_running() {
        log(format_args!(
            "Found installer process. Ignoring app removal"
        ));
        return Ok(());
    } else {
        log(format_args!(
            "Did not find installer process. Running uninstaller"
        ));
    }

    tokio::fs::write(UNINSTALL_SCRIPT_PATH, UNINSTALL_SCRIPT)
        .await
        .context("Failed to write uninstall script")?;

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

/// Figure out if a Mullvad installer is active
fn mullvad_installer_is_running() -> bool {
    let Ok(pids) = list_pids() else {
        // If we can't retrieve any PIDs, assume installer isn't running
        return false;
    };
    pids.into_iter()
        .any(|pid| process_has_mullvad_installer(pid).unwrap_or(false))
}

/// Figure out if the 'pid' process is privileged and has a file open that matches a Mullvad pkg
fn process_has_mullvad_installer(pid: pid_t) -> io::Result<bool> {
    // Ignore process if it isn't running as root
    // This is because the filename is easily spoofable
    if process_bsdinfo(pid)?.pbi_uid != 0 {
        return Ok(false);
    }

    // We're only interested in installer processes
    let process_path = process_path(pid)?;
    if !process_path.starts_with("/System")
        || process_path.file_name() != Some(OsStr::new("installd"))
    {
        return Ok(false);
    }

    // Figure out if one of the file descriptors refers to a Mullvad installer
    for fd in process_file_descriptors(pid)? {
        // Only check vnodes
        if fd.proc_fdtype != PROX_FDTYPE_VNODE as u32 {
            continue;
        }

        let Ok(path) = get_file_desc_vnode_path(pid, &fd) else {
            continue;
        };

        // Check if file refers to a Mullvad .pkg
        let lower_path = path.to_bytes().to_ascii_lowercase();
        let is_pkg = lower_path.ends_with(b".pkg");
        let seq_to_find = b"mullvad";

        if is_pkg
            && lower_path
                .windows(seq_to_find.len())
                .any(|seq| seq == &seq_to_find[..])
        {
            return Ok(true);
        }
    }
    Ok(false)
}
