#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::cmp::{Ord, Ordering, PartialOrd};

lazy_static::lazy_static! {
    static ref STABLE_REGEX: Regex = Regex::new(r"^(\d{4})\.(\d+)$").unwrap();
    static ref BETA_REGEX: Regex = Regex::new(r"^(\d{4})\.(\d+)-beta(\d+)$").unwrap();
}


/// AppVersionInfo represents the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct AppVersionInfo {
    /// False if Mullvad has stopped supporting the currently running version. This could mean
    /// a number of things. For example:
    /// * API endpoints it uses might not work any more.
    /// * Software bundled with this version, such as OpenVPN or OpenSSL, has known security
    ///   issues, so using it is no longer recommended.
    /// The user should really upgrade when this is false.
    pub supported: bool,
    /// Latest stable version
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub latest_stable: AppVersion,
    /// Equal to `latest_stable` when the newest release is a stable release. But will contain
    /// beta versions when those are out for testing.
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub latest_beta: AppVersion,
    /// Whether should update to newer version
    pub suggested_upgrade: Option<AppVersion>,
}

pub type AppVersion = String;


/// Parses a version string into a type that can be used for comparisons.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ParsedAppVersion {
    Stable(u32, u32),
    Beta(u32, u32, u32),
}

impl ParsedAppVersion {
    pub fn from_str(version: &str) -> Option<Self> {
        let get_int = |cap: &regex::Captures<'_>, idx| cap.get(idx)?.as_str().parse().ok();

        if let Some(caps) = STABLE_REGEX.captures(version) {
            let year = get_int(&caps, 1)?;
            let version = get_int(&caps, 2)?;
            Some(Self::Stable(year, version))
        } else if let Some(caps) = BETA_REGEX.captures(version) {
            let year = get_int(&caps, 1)?;
            let version = get_int(&caps, 2)?;
            let beta_version = get_int(&caps, 3)?;
            Some(Self::Beta(year, version, beta_version))
        } else {
            None
        }
    }
}

impl Ord for ParsedAppVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        use ParsedAppVersion::*;
        match (self, other) {
            (Stable(year, version), Stable(other_year, other_version)) => {
                year.cmp(other_year).then(version.cmp(other_version))
            }
            // A stable version of the same year and version is always greater than a beta
            (Stable(year, version), Beta(other_year, other_version, _)) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(Ordering::Greater),
            (
                Beta(year, version, beta_version),
                Beta(other_year, other_version, other_beta_version),
            ) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(beta_version.cmp(other_beta_version)),
            (Beta(year, version, _beta_version), Stable(other_year, other_version)) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(Ordering::Less),
        }
    }
}

impl PartialOrd for ParsedAppVersion {
    fn partial_cmp(&self, other: &ParsedAppVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ToString for ParsedAppVersion {
    fn to_string(&self) -> String {
        match self {
            Self::Stable(year, version) => format!("{}.{}", year, version),
            Self::Beta(year, version, beta_version) => {
                format!("{}.{}-beta{}", year, version, beta_version)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_regex() {
        assert!(STABLE_REGEX.is_match("2020.4"));
        assert!(!STABLE_REGEX.is_match("2020.4-beta3"));
        assert!(BETA_REGEX.is_match("2020.4-beta3"));
        assert!(!STABLE_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(!STABLE_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.4"));
    }

    #[test]
    fn test_version_parsing() {
        let tests = vec![
            ("2020.4", Some(ParsedAppVersion::Stable(2020, 4))),
            ("2020.4-beta3", Some(ParsedAppVersion::Beta(2020, 4, 3))),
            ("2020.15-beta1-dev-f16be4", None),
            ("2020.15-dev-f16be4", None),
            ("", None),
        ];

        for (input, expected_output) in tests {
            assert_eq!(ParsedAppVersion::from_str(&input), expected_output,);
        }
    }
}
