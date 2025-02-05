use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Formatter},
    str::FromStr,
    sync::LazyLock,
};

static STABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{4})\.(\d+)$").unwrap());

// The `BETA_REGEX` and `DEV_REGEX` both support parsing -alpha and -beta versions.
// However, the -alpha version is currently only used by Android to tag internal
// alpha builds, so the code in this file will simply parse alpha versions as beta versions, so
// as far as the Rust code is concerned there is no difference between -alpha and -beta.
static BETA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)                     # Enable insignification whitespace mode
        ^                           # Start of string
        (\d{4})                     # Year
        \.                          # Literal dot separator
        (\d+)                       # Version number
        -                           # Literal -
        (?:alpha|beta)              # Alpha or beta (non-capturing group)
        (\d+)                       # Alpha or beta version
        $                           # End of string
        "#,
    )
    .unwrap()
});

static DEV_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)                     # Enable insignificant whitespace mode
        ^                           # Start of string
        (\d{4})                     # Year
        \.                          # Literal dot separator
        (\d+)                       # Version number
        (\.\d+)?                    # Optional patch number
        (-                          # Literal -
            (?:alpha|beta)          # Alpha or beta (non-capturing group)
            (\d+)                   # Alpha or beta version
        )?
        -dev-
        (\w+)                       # Commit hash
        $                           # End of string
        "#,
    )
    .unwrap()
});

/// AppVersionInfo represents the current stable and the current latest release versions of the
/// Mullvad VPN app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppVersionInfo {
    /// False if Mullvad has stopped supporting the currently running version. This could mean
    /// a number of things. For example:
    /// * API endpoints it uses might not work any more.
    /// * Software bundled with this version, such as OpenVPN or OpenSSL, has known security
    ///   issues, so using it is no longer recommended.
    ///
    /// The user should really upgrade when this is false.
    pub supported: bool,
    /// Latest stable version
    pub latest_stable: AppVersion,
    /// Equal to `latest_stable` when the newest release is a stable release. But will contain
    /// beta versions when those are out for testing.
    pub latest_beta: AppVersion,
    /// Whether should update to newer version
    pub suggested_upgrade: Option<AppVersion>,
}

pub type AppVersion = String;

/// Parses a version string into a type that can be used for comparisons.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ParsedAppVersion {
    Stable(u32, u32),
    Beta(u32, u32, u32),
    Dev(u32, u32, Option<u32>, String),
}

impl FromStr for ParsedAppVersion {
    type Err = ();
    fn from_str(version: &str) -> Result<Self, Self::Err> {
        let get_int = |cap: &regex::Captures<'_>, idx| cap.get(idx)?.as_str().parse().ok();

        if let Some(caps) = STABLE_REGEX.captures(version) {
            let year = get_int(&caps, 1).ok_or(())?;
            let version = get_int(&caps, 2).ok_or(())?;
            Ok(Self::Stable(year, version))
        } else if let Some(caps) = BETA_REGEX.captures(version) {
            let year = get_int(&caps, 1).ok_or(())?;
            let version = get_int(&caps, 2).ok_or(())?;
            let beta_version = get_int(&caps, 3).ok_or(())?;
            Ok(Self::Beta(year, version, beta_version))
        } else if let Some(caps) = DEV_REGEX.captures(version) {
            let year = get_int(&caps, 1).ok_or(())?;
            let version = get_int(&caps, 2).ok_or(())?;
            let beta_version = caps.get(4).map(|_| get_int(&caps, 5).unwrap());
            let dev_hash = caps.get(6).ok_or(())?.as_str().to_string();
            Ok(Self::Dev(year, version, beta_version, dev_hash))
        } else {
            Err(())
        }
    }
}

impl ParsedAppVersion {
    pub fn is_dev(&self) -> bool {
        matches!(self, ParsedAppVersion::Dev(..))
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
            // We assume that a dev version of the same year and version is newer
            (Stable(year, version), Dev(other_year, other_version, ..)) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(Ordering::Less),

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
            // We assume that a dev version of the same year and version is newer
            (Beta(year, version, _), Dev(other_year, other_version, ..)) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(Ordering::Less),

            // Dev versions of the same year and version are assumed to be equal
            (Dev(year, version, ..), Dev(other_year, other_version, ..)) => {
                year.cmp(other_year).then(version.cmp(other_version))
            }
            (Dev(year, version, ..), Stable(other_year, other_version)) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(Ordering::Greater),
            (Dev(year, version, ..), Beta(other_year, other_version, _)) => year
                .cmp(other_year)
                .then(version.cmp(other_version))
                .then(Ordering::Greater),
        }
    }
}

impl PartialOrd for ParsedAppVersion {
    fn partial_cmp(&self, other: &ParsedAppVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for ParsedAppVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable(year, version) => write!(f, "{year}.{version}"),
            Self::Beta(year, version, beta_version) => {
                write!(f, "{year}.{version}-beta{beta_version}")
            }
            Self::Dev(year, version, beta_version, hash) => {
                if let Some(beta_version) = beta_version {
                    write!(f, "{year}.{version}-beta{beta_version}-dev-{hash}")
                } else {
                    write!(f, "{year}.{version}-dev-{hash}")
                }
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
        assert!(!STABLE_REGEX.is_match("2020.4-alpha3"));
        assert!(BETA_REGEX.is_match("2020.4-beta3"));
        assert!(BETA_REGEX.is_match("2020.4-alpha99"));
        assert!(!STABLE_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(!STABLE_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.5-alpha2-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(!BETA_REGEX.is_match("2020.4"));
        assert!(DEV_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(DEV_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(DEV_REGEX.is_match("2020.5-alpha4-dev-f16be4"));
        assert!(!DEV_REGEX.is_match("2020.5"));
        assert!(!DEV_REGEX.is_match("2020.5-beta1"));
        assert!(!DEV_REGEX.is_match("2020.5-alpha1"));
    }

    #[test]
    fn test_version_parsing() {
        let tests = vec![
            ("2020.4", Some(ParsedAppVersion::Stable(2020, 4))),
            ("2020.4-beta3", Some(ParsedAppVersion::Beta(2020, 4, 3))),
            (
                "2020.15-beta1-dev-f16be4",
                Some(ParsedAppVersion::Dev(
                    2020,
                    15,
                    Some(1),
                    "f16be4".to_string(),
                )),
            ),
            (
                "2020.15-dev-f16be4",
                Some(ParsedAppVersion::Dev(2020, 15, None, "f16be4".to_string())),
            ),
            ("2020.15-9000", None),
            ("", None),
        ];

        for (input, expected_output) in tests {
            assert_eq!(ParsedAppVersion::from_str(input).ok(), expected_output,);
        }
    }

    #[test]
    fn test_alpha_version_string_is_parsed_as_beta() {
        assert_eq!(
            ParsedAppVersion::from_str("2020.4-alpha13").unwrap(),
            ParsedAppVersion::from_str("2020.4-beta13").unwrap(),
        );
    }
}
