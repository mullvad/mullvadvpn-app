use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use crate::PreStableType::{Alpha, Beta};
use regex::Regex;

/// The Mullvad VPN app product version
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    /// The last two digits of the version's year
    pub year: String,
    pub incremental: String,
    /// A version can have an optional pre-stable type, e.g. alpha or beta. If `pre_stable`
    /// and `dev` both are None the version is stable.
    pub pre_stable: Option<PreStableType>,
    /// All versions may have an optional -dev-[commit hash] suffix.
    pub dev: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreStableType {
    Alpha(String),
    Beta(String),
}

impl Version {
    pub fn parse(version: &str) -> Version {
        Version::from_str(version).unwrap()
    }

    pub fn is_stable(&self) -> bool {
        self.pre_stable.is_none() && self.dev.is_none()
    }

    pub fn alpha(&self) -> Option<&str> {
        match &self.pre_stable {
            Some(PreStableType::Alpha(v)) => Some(v),
            _ => None,
        }
    }

    pub fn beta(&self) -> Option<&str> {
        match &self.pre_stable {
            Some(PreStableType::Beta(beta)) => Some(beta),
            _ => None,
        }
    }
}

impl Display for Version {
    /// Format Version as a string: year.incremental-{alpha|beta}-{dev}
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Version {
            year,
            incremental,
            pre_stable,
            dev,
        } = &self;

        write!(f, "20{year}.{incremental}")?;

        match pre_stable {
            Some(PreStableType::Alpha(version)) => write!(f, "-alpha{version}")?,
            Some(PreStableType::Beta(version)) => write!(f, "-beta{version}")?,
            None => (),
        };

        if let Some(commit_hash) = dev {
            write!(f, "-dev-{commit_hash}")?;
        }

        Ok(())
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?x)                             # enable insignificant whitespace mode
                20(?<year>\d{2})\.                 # the last two digits of the year
                (?<incremental>[1-9]\d?)           # the incrementing version number
                (?:                                # (optional) alpha or beta or dev
                  -alpha(?<alpha>[1-9]\d?\d?)|
                  -beta(?<beta>[1-9]\d?\d?)
                )?
                (?:
                  -dev-(?<dev>[0-9a-f]+)
                )?$
                ",
            )
            .unwrap()
        });

        let captures = VERSION_REGEX
            .captures(version)
            .ok_or_else(|| format!("Version does not match expected format: {version}"))?;

        let year = captures
            .name("year")
            .expect("Missing year")
            .as_str()
            .to_owned();

        let incremental = captures
            .name("incremental")
            .ok_or("Missing incremental")?
            .as_str()
            .to_owned();

        let alpha = captures.name("alpha").map(|m| m.as_str().to_owned());
        let beta = captures.name("beta").map(|m| m.as_str().to_owned());
        let dev = captures.name("dev").map(|m| m.as_str().to_owned());

        let pre_stable = match (alpha, beta) {
            (None, None) => None,
            (Some(v), None) => Some(Alpha(v)),
            (None, Some(v)) => Some(Beta(v)),
            _ => return Err(format!("Invalid version: {version}")),
        };

        Ok(Version {
            year,
            incremental,
            pre_stable,
            dev,
        })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let version = "2021.34";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "21");
        assert_eq!(parsed.incremental, "34");
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev, None);
        assert!(parsed.is_stable());
    }

    #[test]
    fn test_parse_with_alpha() {
        let version = "2023.1-alpha77";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "23");
        assert_eq!(parsed.incremental, "1");
        assert_eq!(parsed.alpha(), Some("77"));
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev, None);
        assert!(!parsed.is_stable());

        let version = "2021.34-alpha777";
        let parsed = Version::parse(version);
        assert_eq!(parsed.alpha(), Some("777"));
    }

    #[test]
    fn test_parse_with_beta() {
        let version = "2021.34-beta5";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "21");
        assert_eq!(parsed.incremental, "34");
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), Some("5"));
        assert_eq!(parsed.dev, None);
        assert!(!parsed.is_stable());

        let version = "2021.34-beta453";
        let parsed = Version::parse(version);
        assert_eq!(parsed.beta(), Some("453"));
    }

    #[test]
    fn test_parse_with_dev() {
        let version = "2021.34-dev-0b60e4d87";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "21");
        assert_eq!(parsed.incremental, "34");
        assert!(!parsed.is_stable());
        assert_eq!(parsed.dev, Some("0b60e4d87".to_string()));
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), None);
    }

    #[test]
    fn test_parse_both_beta_and_dev() {
        let version = "2024.8-beta1-dev-e5483d";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "24");
        assert_eq!(parsed.incremental, "8");
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), Some("1"));
        assert_eq!(parsed.dev, Some("e5483d".to_string()));
        assert!(!parsed.is_stable());
    }

    #[test]
    #[should_panic]
    fn test_panics_on_invalid_version() {
        Version::parse("2021");
    }

    #[test]
    #[should_panic]
    fn test_panics_on_invalid_version_type_number() {
        Version::parse("2021.1-beta001");
    }

    #[test]
    #[should_panic]
    fn test_panics_on_alpha_and_beta_in_same_version() {
        Version::parse("2021.1-beta5-alpha2");
    }

    #[test]
    #[should_panic]
    fn test_panics_on_dev_without_commit_hash() {
        Version::parse("2021.1-dev");
    }

    #[test]
    fn test_version_display() {
        let version = "2024.8-beta1-dev-e5483d";
        let parsed = Version::parse(version);

        assert_eq!(format!("{parsed}"), version);
    }
}
