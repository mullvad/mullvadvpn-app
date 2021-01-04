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

    format!("Linux {}", version)
}

pub fn short_version() -> String {
    version()
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
    std::iter::empty()
}
