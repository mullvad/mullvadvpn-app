use std::process::Command;

#[cfg(target_os = "linux")]
mod imp {
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

        format!("Linux {}", version)
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
        super::command_stdout_lossy("lsb_release", &["-ds"]).and_then(|output| {
            if output.is_empty() {
                None
            } else {
                Some(output)
            }
        })
    }

    pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
        std::iter::empty()
    }
}

#[cfg(target_os = "macos")]
mod imp {
    pub fn version() -> String {
        format!(
            "macOS {}",
            super::command_stdout_lossy("sw_vers", &["-productVersion"])
                .unwrap_or(String::from("[Failed to detect version]"))
        )
    }

    pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
        std::iter::empty()
    }
}

#[cfg(windows)]
mod imp {
    pub fn version() -> String {
        let system_info =
            super::command_stdout_lossy("systeminfo", &["/FO", "LIST"]).unwrap_or_else(String::new);

        let mut version = None;
        let mut full_version = None;

        for info_line in system_info.lines() {
            let mut info_parts = info_line.split(":");

            match info_parts.next() {
                Some("OS Name") => {
                    version = info_parts
                        .next()
                        .map(|s| s.trim().trim_start_matches("Microsoft Windows "))
                }
                Some("OS Version") => full_version = info_parts.next().map(str::trim),
                _ => {}
            }
        }

        let version = version.unwrap_or("N/A");
        let full_version = full_version.unwrap_or("N/A");
        format!("Windows {} ({})", version, full_version)
    }

    pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
        std::iter::empty()
    }
}

#[cfg(target_os = "android")]
mod imp {
    use std::collections::HashMap;

    pub fn version() -> String {
        let version = get_prop("ro.build.version.release").unwrap_or_else(|| "N/A".to_owned());
        let api_level = get_prop("ro.build.version.sdk").unwrap_or_else(|| "N/A".to_owned());

        let manufacturer =
            get_prop("ro.product.manufacturer").unwrap_or_else(|| "Unknown brand".to_owned());
        let product = get_prop("ro.product.model").unwrap_or_else(|| "Unknown model".to_owned());

        format!(
            "Android {} (API: {}) - {} {}",
            version, api_level, manufacturer, product
        )
    }

    pub fn extra_metadata() -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "abi".to_owned(),
            get_prop("ro.product.cpu.abilist").unwrap_or_else(|| "N/A".to_owned()),
        );
        metadata
    }

    fn get_prop(property: &str) -> Option<String> {
        super::command_stdout_lossy("getprop", &[property])
    }
}

/// Helper for getting stdout of some command as a String. Ignores the exit code of the command.
fn command_stdout_lossy(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .ok()
}

pub use imp::{extra_metadata, version};
