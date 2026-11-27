use tokio::{fs, process::Command};

use crate::command::command_stdout_lossy;

pub fn version() -> String {
    // The OS version information is obtained first from the os-release file. If that
    // information is incomplete or unavailable, an attempt is made to obtain the
    // version information from the lsb_release command. If that fails, any partial
    // information from os-release is used if available, or a fallback message if
    // reading from the os-release file produced
    // no version information.
    let version = read_os_release_file().unwrap_or_else(|incomplete_info| {
        parse_lsb_release().unwrap_or_else(|| {
            incomplete_info.unwrap_or_else(|| String::from("[Failed to detect version]"))
        })
    });

    format!("Linux {version}")
}

pub fn short_version() -> String {
    let version = read_os_release_file_short().unwrap_or_else(|| {
        parse_lsb_release().unwrap_or_else(|| String::from("[Failed to detect version]"))
    });

    format!("Linux {version}")
}

// Note: This function performs blocking I/O.
fn read_os_release_file_short() -> Option<String> {
    let mut os_release_info = rs_release::get_os_release().ok()?;
    let os_name = os_release_info.remove("NAME");
    let os_version_id = os_release_info.remove("VERSION_ID");

    if let Some(os_name) = os_name
        && os_name != "NixOS"
        && let Some(os_version_id) = os_version_id
    {
        return Some(format!("{os_name} {os_version_id}"));
    }

    os_release_info.remove("PRETTY_NAME")
}

// Note: This function performs blocking I/O.
fn read_os_release_file() -> Result<String, Option<String>> {
    let mut os_release_info = rs_release::get_os_release().map_err(|_| None)?;
    let os_name = os_release_info.remove("NAME");
    let os_version = os_release_info.remove("VERSION");

    if os_name.is_some() || os_version.is_some() {
        let full_info_available = os_name.is_some() && os_version.is_some();

        let gathered_info = format!(
            "{} {}",
            os_name.unwrap_or_else(|| "[unknown distribution]".to_owned()),
            os_version.unwrap_or_else(|| "[unknown version]".to_owned())
        );

        if full_info_available {
            Ok(gathered_info)
        } else {
            // Partial version information
            Err(Some(gathered_info))
        }
    } else {
        // No information was obtained
        Err(None)
    }
}

fn parse_lsb_release() -> Option<String> {
    command_stdout_lossy("lsb_release", &["-ds"])
        .ok()
        .filter(|output| !output.is_empty())
}

pub async fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    let metadata = [
        kernel_version().await,
        wg_version().await,
        systemd_version().await,
    ];
    let network_manager = cfg_select! {
        feature = "network-manager" => nm_version().await,
        _ => None,
    };

    metadata.into_iter().chain([network_manager]).flatten()
}

/// `uname -r` outputs a single line containing only the kernel version:
/// > 5.9.15
async fn kernel_version() -> Option<(String, String)> {
    let out = Command::new("uname").arg("-r").output().await.ok()?;
    let kernel = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Some(("kernel".to_string(), kernel))
}

/// NetworkManager's version is returned as a numeric version string
/// > 1.26.0
#[cfg(feature = "network-manager")]
async fn nm_version() -> Option<(String, String)> {
    let nm = talpid_dbus::network_manager::NetworkManagerConnection::new()
        .await
        .ok()?;
    Some(("nm".to_string(), nm.version_string().await.ok()?))
}

/// `/sys/module/wireguard/version` contains only a numeric version string
/// > 1.0.0
async fn wg_version() -> Option<(String, String)> {
    let wireguard_version = fs::read_to_string("/sys/module/wireguard/version")
        .await
        .ok()?
        .trim()
        .to_string();
    Some(("wireguard".to_string(), wireguard_version))
}

/// `systemctl --version` usually outputs two lines - one with the version, and another listing
/// features:
/// > systemd 246 (246)
/// > +PAM +AUDIT -SELINUX +IMA +APPARMOR +SMACK -SYSVINIT +UTMP +LIBCRYPTSETUP +GCRYPT -GNUTLS +ACL
async fn systemd_version() -> Option<(String, String)> {
    let out = Command::new("systemctl")
        .arg("--version")
        .output()
        .await
        .ok()?;
    let systemd = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let systemd = systemd.lines().next()?.to_string();
    Some(("systemd".to_string(), systemd))
}
