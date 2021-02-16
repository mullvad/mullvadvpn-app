mod command;
use command::command_stdout_lossy;

pub fn version() -> String {
    let version = run_sw_vers().unwrap_or(String::from("N/A"));
    format!("macOS {}", version)
}


pub fn short_version() -> String {
    let version = run_sw_vers()
        .and_then(parse_short_version_output)
        .map(|(major, minor)| format!("{}.{}", major, minor))
        .unwrap_or(String::from("N/A"));
    format!("macOS {}", version)
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    std::iter::empty()
}

/// Outputs a string in a format `$major.$minor.$patch`, e.g. `11.0.1`
fn run_sw_vers() -> Option<String> {
    command_stdout_lossy("sw_vers", &["-productVersion"])
}

fn parse_short_version_output(output: String) -> Option<(u32, u32)> {
    let mut parts = output.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}
