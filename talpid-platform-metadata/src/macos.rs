mod command;

use command::command_stdout_lossy;
use std::io;

pub fn version() -> String {
    let version = MacosVersion::new()
        .map(|version| version.version())
        .unwrap_or(String::from("N/A"));
    format!("macOS {}", version)
}

pub fn short_version() -> String {
    let version = MacosVersion::new()
        .map(|version| version.short_version())
        .unwrap_or(String::from("N/A"));
    format!("macOS {}", version)
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    std::iter::empty()
}

#[derive(Debug, PartialEq)]
pub struct MacosVersion {
    raw_version: String,
    major: u32,
    minor: u32,
    patch: u32,
}

impl MacosVersion {
    pub fn new() -> Result<MacosVersion, io::Error> {
        Self::from_raw_version(&run_sw_vers()?)
    }

    fn from_raw_version(version_string: &str) -> Result<MacosVersion, io::Error> {
        let (major, minor, patch) = parse_version_output(version_string).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Failed to parse raw version string",
        ))?;
        Ok(MacosVersion {
            raw_version: version_string.to_owned(),
            major,
            minor,
            patch,
        })
    }

    /// Return the current version as a string (e.g. 14.2.1)
    pub fn version(&self) -> String {
        self.raw_version.clone()
    }

    /// Return the current version as a string (e.g. 14.2), not including the patch version
    pub fn short_version(&self) -> String {
        format!("{}.{}", self.major_version(), self.minor_version())
    }

    pub fn major_version(&self) -> u32 {
        self.major
    }

    pub fn minor_version(&self) -> u32 {
        self.minor
    }

    pub fn patch_version(&self) -> u32 {
        self.patch
    }
}

/// Outputs a string in a format `$major.$minor.$patch`, e.g. `11.0.1`
fn run_sw_vers() -> io::Result<String> {
    command_stdout_lossy("sw_vers", &["-productVersion"])
}

fn parse_version_output(output: &str) -> Option<(u32, u32, u32)> {
    let mut parts = output.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts
        .next()
        .and_then(|patch| patch.parse().ok())
        .unwrap_or(0);
    Some((major, minor, patch))
}

#[test]
fn test_version_parsing() {
    // % sw_vers --productVersion
    // 14.2.1
    let version = MacosVersion::from_raw_version("14.2.1").expect("failed to parse version");
    assert_eq!(version.major_version(), 14);
    assert_eq!(version.minor_version(), 2);
    assert_eq!(version.patch_version(), 1);
}
