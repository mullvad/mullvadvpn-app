use std::{collections::BTreeMap, process::Command};


pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

pub fn collect() -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("id".to_owned(), uuid::Uuid::new_v4().to_string());
    metadata.insert(
        "mullvad-product-version".to_owned(),
        PRODUCT_VERSION.to_owned(),
    );
    metadata.insert("os".to_owned(), os::version());
    metadata
}

#[cfg(target_os = "linux")]
mod os {
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
}

#[cfg(target_os = "macos")]
mod os {
    pub fn version() -> String {
        format!(
            "macOS {}",
            super::command_stdout_lossy("sw_vers", &["-productVersion"])
                .unwrap_or(String::from("[Failed to detect version]"))
        )
    }
}

#[cfg(windows)]
mod os {
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
}

#[cfg(target_os = "android")]
mod os {
    pub fn version() -> String {
        let version = get_prop("ro.build.version.release").unwrap_or_else(String::new);
        let api_level = get_prop("ro.build.version.sdk")
            .map(|api| format!(" (API level: {})", api))
            .unwrap_or_else(String::new);
        let abi_list = get_prop("ro.product.cpu.abilist")
            .map(|abis| format!(" (ABI list: {})", abis))
            .unwrap_or_else(String::new);

        let manufacturer = get_prop("ro.product.manufacturer").unwrap_or_default();
        let product = get_prop("ro.product.model").unwrap_or_default();
        let build = get_prop("ro.build.display.id").unwrap_or_default();

        format!(
            "Android {}{}{} - {} {} {}",
            version, api_level, abi_list, manufacturer, product, build
        )
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
