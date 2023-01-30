mod command;
use command::command_stdout_lossy;

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

fn read_os_release_file_short() -> Option<String> {
    let mut os_release_info = rs_release::get_os_release().ok()?;
    let os_name = os_release_info.remove("NAME");
    let os_version_id = os_release_info.remove("VERSION_ID");

    if let Some(os_name) = os_name {
        if os_name != "NixOS" {
            if let Some(os_version_id) = os_version_id {
                return Some(format!("{os_name} {os_version_id}"));
            }
        }
    }

    os_release_info.remove("PRETTY_NAME")
}

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
    command_stdout_lossy("lsb_release", &["-ds"]).and_then(|output| {
        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    })
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    [kernel_version, nm_version, wg_version, systemd_version]
        .iter()
        .filter_map(|f| f())
}

/// `uname -r` outputs a single line containing only the kernel version:
/// > 5.9.15
fn kernel_version() -> Option<(String, String)> {
    let kernel = command_stdout_lossy("uname", &["-r"])?;
    Some(("kernel".to_string(), kernel))
}

/// NetworkManager's version is returned as a numeric version string
/// > 1.26.0
fn nm_version() -> Option<(String, String)> {
    let nm = talpid_dbus::network_manager::NetworkManager::new().ok()?;
    Some(("nm".to_string(), nm.version_string().ok()?))
}

/// `/sys/module/wireguard/version` contains only a numeric version string
/// > 1.0.0
fn wg_version() -> Option<(String, String)> {
    let wireguard_version = std::fs::read_to_string("/sys/module/wireguard/version")
        .ok()?
        .trim()
        .to_string();
    Some(("wireguard".to_string(), wireguard_version))
}

/// `systemctl --version` usually outpus two lines - one with the version, and another listing
/// features:
/// > systemd 246 (246)
/// > +PAM +AUDIT -SELINUX +IMA +APPARMOR +SMACK -SYSVINIT +UTMP +LIBCRYPTSETUP +GCRYPT -GNUTLS +ACL
fn systemd_version() -> Option<(String, String)> {
    let systemd_version_output = command_stdout_lossy("systemctl", &["--version"])?;
    let version = systemd_version_output.lines().next()?.to_string();
    Some(("systemd".to_string(), version))
}
