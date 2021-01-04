mod command;
use command::command_stdout_lossy;

pub fn version() -> String {
    format!(
        "macOS {}",
        command_stdout_lossy("sw_vers", &["-productVersion"])
            .unwrap_or(String::from("[Failed to detect version]"))
    )
}

pub fn short_version() -> String {
    version()
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    std::iter::empty()
}
