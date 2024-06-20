mod command;

use command::command_stdout_lossy;
use std::{cmp::Ordering, fmt, fmt::Formatter, io};

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

#[derive(Debug, Clone)]
pub struct MacosVersion {
    raw_version: String,
    major: u32,
    minor: u32,
    patch: Option<u32>,
}

impl PartialEq for MacosVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major_version() == other.major_version()
            && self.minor_version() == other.minor_version()
            && self.patch_version() == other.patch_version()
    }
}

impl PartialOrd for MacosVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let major = self.major_version().partial_cmp(&other.major_version())?;
        let minor = self.minor_version().partial_cmp(&other.minor_version())?;
        let patch = self.patch_version().partial_cmp(&other.patch_version())?;
        Some(major.then(minor).then(patch))
    }
}

impl fmt::Display for MacosVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.version())
    }
}

impl MacosVersion {
    pub fn new() -> Result<MacosVersion, io::Error> {
        Self::from_raw_version(&run_sw_vers()?)
    }

    pub fn from_raw_version(version_string: &str) -> Result<MacosVersion, io::Error> {
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
        self.patch.unwrap_or(0)
    }
}

/// Outputs a string in a format `$major.$minor.$patch`, e.g. `11.0.1`
fn run_sw_vers() -> io::Result<String> {
    command_stdout_lossy("sw_vers", &["-productVersion"])
}

fn parse_version_output(output: &str) -> Option<(u32, u32, Option<u32>)> {
    let mut parts = output.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().and_then(|patch| patch.parse().ok());
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

#[test]
fn test_version_order() {
    assert_eq!(
        MacosVersion::from_raw_version("13.0").unwrap(),
        MacosVersion::from_raw_version("13.0.0").unwrap()
    );

    assert_eq!(
        MacosVersion::from_raw_version("13.0")
            .unwrap()
            .partial_cmp(&MacosVersion::from_raw_version("13.0.0").unwrap()),
        Some(Ordering::Equal),
    );

    // test major version
    assert!(
        MacosVersion::from_raw_version("13.0").unwrap()
            < MacosVersion::from_raw_version("14.2.1").unwrap()
    );
    assert!(
        MacosVersion::from_raw_version("13.0").unwrap()
            > MacosVersion::from_raw_version("12.1").unwrap()
    );

    // test minor version
    assert!(
        MacosVersion::from_raw_version("14.3").unwrap()
            > MacosVersion::from_raw_version("14.2").unwrap()
    );
    assert!(
        MacosVersion::from_raw_version("14.2").unwrap()
            < MacosVersion::from_raw_version("14.3").unwrap()
    );

    // test patch version
    assert!(
        MacosVersion::from_raw_version("14.2.1").unwrap()
            > MacosVersion::from_raw_version("14.2").unwrap()
    );
    assert!(
        MacosVersion::from_raw_version("14.2.2").unwrap()
            > MacosVersion::from_raw_version("14.2.1").unwrap()
    );
    assert!(
        MacosVersion::from_raw_version("14.2.2").unwrap()
            < MacosVersion::from_raw_version("14.2.3").unwrap()
    );
}
