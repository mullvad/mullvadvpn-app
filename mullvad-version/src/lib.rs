use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use regex::Regex;

/// The Mullvad VPN app product version
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub year: String,
    pub incremental: String,
    pub version_type: VersionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionType {
    Alpha(String),
    Beta(String),
    Dev(String),
    Stable,
}

impl Version {
    pub fn parse(version: &str) -> Version {
        Version::from_str(version).unwrap()
    }

    pub fn is_stable(&self) -> bool {
        matches!(&self.version_type, VersionType::Stable)
    }

    pub fn alpha(&self) -> Option<&str> {
        match &self.version_type {
            VersionType::Alpha(v) => Some(v),
            _ => None,
        }
    }

    pub fn beta(&self) -> Option<&str> {
        match &self.version_type {
            VersionType::Beta(v) => Some(v),
            _ => None,
        }
    }

    pub fn dev(&self) -> Option<&str> {
        match &self.version_type {
            VersionType::Dev(v) => Some(v),
            _ => None,
        }
    }
}

impl Display for Version {
    /// Format Version as a string: year.incremental-{alpha|beta|dev}
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Version {
            year,
            incremental,
            version_type,
        } = &self;

        write!(f, "{year}.{incremental}")?;

        match version_type {
            VersionType::Alpha(version) => write!(f, "-alpha{version}"),
            VersionType::Beta(version) => write!(f, "-beta{version}"),
            VersionType::Dev(commit_hash) => write!(f, "-dev-{commit_hash}"),
            VersionType::Stable => Ok(()),
        }
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?x)
                20(?<year>\d{2})\.                 # the last two digits of the year
                (?<incremental>[1-9]\d?)           # the incrementing version number
                (?:                                # (optional) alpha or beta or dev
                  -alpha(?<alpha>[1-9]\d?\d?)|
                  -beta(?<beta>[1-9]\d?\d?)|
                  -dev-(?<dev>[0-9a-f]+)|
                  (?<no_ver_dev>-dev)
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

        let alpha = captures.name("alpha").map(|m| m.as_str());
        let beta = captures.name("beta").map(|m| m.as_str());
        let dev = captures
            .name("dev")
            .map(|m| m.as_str())
            .or(captures.name("no_ver_dev").map(|_| ""));

        let sub_type = match (alpha, beta, dev) {
            (Some(v), _, _) => VersionType::Alpha(v.to_owned()),
            (_, Some(v), _) => VersionType::Beta(v.to_owned()),
            (_, _, Some(v)) => VersionType::Dev(v.to_owned()),
            (None, None, None) => VersionType::Stable,
        };

        Ok(Version {
            year,
            incremental,
            version_type: sub_type,
        })
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
        assert_eq!(parsed.dev(), None);
        assert_eq!(parsed.is_stable(), true);
    }

    #[test]
    fn test_parse_with_alpha() {
        let version = "2023.1-alpha77";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "23");
        assert_eq!(parsed.incremental, "1");
        assert_eq!(parsed.alpha(), Some("77"));
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev(), None);
        assert_eq!(parsed.is_stable(), false);

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
        assert_eq!(parsed.dev(), None);
        assert_eq!(parsed.is_stable(), false);

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
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev(), Some("0b60e4d87"));
        assert_eq!(parsed.is_stable(), false);
    }

    #[test]
    fn test_parse_with_dev_no_version() {
        let version = "2099.99-dev";
        let parsed = Version::parse(version);
        assert_eq!(parsed.year, "99");
        assert_eq!(parsed.incremental, "99");
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev(), Some(""));
        assert_eq!(parsed.is_stable(), false);
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
}
