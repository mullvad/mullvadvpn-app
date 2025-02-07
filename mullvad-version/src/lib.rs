use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use regex_lite::Regex;

/// The Mullvad VPN app product version
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub year: u32,
    pub incremental: u32,
    /// A version can have an optional pre-stable type, e.g. alpha or beta.
    pub pre_stable: Option<PreStableType>,
    /// All versions may have an optional -dev-[commit hash] suffix.
    pub dev: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreStableType {
    Alpha(u32),
    Beta(u32),
}

impl Ord for PreStableType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PreStableType::Alpha(a), PreStableType::Alpha(b)) => a.cmp(b),
            (PreStableType::Beta(a), PreStableType::Beta(b)) => a.cmp(b),
            (PreStableType::Alpha(_), PreStableType::Beta(_)) => Ordering::Less,
            (PreStableType::Beta(_), PreStableType::Alpha(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for PreStableType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Version {
    /// Returns true if this version has a -dev suffix, e.g. 2025.2-beta1-dev-123abc
    pub fn is_dev(&self) -> bool {
        self.dev.is_some()
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let type_ordering = match (&self.pre_stable, &other.pre_stable) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(self_pre_stable), Some(other_pre_stable)) => {
                self_pre_stable.cmp(other_pre_stable)
            }
        };

        // The dev vs non-dev ordering. For a version of a given type, if all else is equal
        // a dev version is greater than a non-dev version.
        let dev_ordering = match (self.is_dev(), other.is_dev()) {
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (_, _) => None,
        };

        let release_ordering = self
            .year
            .cmp(&other.year)
            .then(self.incremental.cmp(&other.incremental))
            .then(type_ordering);

        match release_ordering {
            Ordering::Equal => dev_ordering,
            _ => Some(release_ordering),
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

        write!(f, "{year}.{incremental}")?;

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

static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)                                     # enable insignificant whitespace mode
                (?<year>\d{4})\.                   # the year
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

impl FromStr for Version {
    type Err = String;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        let captures = VERSION_REGEX
            .captures(version)
            .ok_or_else(|| format!("Version does not match expected format: {version}"))?;

        let year = captures.name("year").unwrap().as_str().parse().unwrap();

        let incremental = captures
            .name("incremental")
            .unwrap()
            .as_str()
            .parse()
            .unwrap();

        let alpha = captures.name("alpha").map(|m| m.as_str().parse().unwrap());
        let beta = captures.name("beta").map(|m| m.as_str().parse().unwrap());
        let dev = captures.name("dev").map(|m| m.as_str().to_owned());

        let pre_stable = match (alpha, beta) {
            (None, None) => None,
            (Some(v), None) => Some(PreStableType::Alpha(v)),
            (None, Some(v)) => Some(PreStableType::Beta(v)),
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
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to parse a version string
    fn parse(version: &str) -> Version {
        version.parse().unwrap()
    }

    #[test]
    fn test_version_ordering() {
        // Test year comparison
        assert!(parse("2022.1") > parse("2021.1"),);

        // Test incremental comparison
        assert!(parse("2021.2") > parse("2021.1"),);

        // Test stable vs pre-release
        assert!(parse("2021.1") > parse("2021.1-beta1"),);
        assert!(parse("2021.1") > parse("2021.1-alpha1"),);

        // Test beta vs alpha
        assert!(parse("2021.1-beta1") > parse("2021.1-alpha1"),);
        assert!(parse("2021.1-beta1") > parse("2021.1-alpha2"),);
        assert!(parse("2021.2-alpha1") > parse("2021.1-beta2"),);

        // Test version numbers within same type
        assert!(parse("2021.1-beta2") > parse("2021.1-beta1"),);
        assert!(parse("2021.1-alpha2") > parse("2021.1-alpha1"),);

        // Test dev versions
        assert!(parse("2021.1-dev-abc") > parse("2021.1"),);
        assert!(parse("2021.2") > parse("2021.1-dev-abc"),);
        assert!(parse("2021.1-dev-abc") > parse("2021.1-beta1"),);
        assert!(parse("2021.1-dev-abc") > parse("2021.1-alpha1"),);
        assert!(parse("2025.1-dev-abc") > parse("2025.1-beta1-dev-abc"),);
        assert!(parse("2025.1-dev-abc") > parse("2025.1-beta2-dev-abc"),);
        assert!(parse("2025.1-dev-abc") > parse("2025.1-alpha2-dev-abc"),);
        assert!(parse("2025.1-beta1-dev-abc") > parse("2025.1-alpha7-dev-abc"),);
        assert!(parse("2025.2-alpha1-dev-abc") > parse("2025.1-beta7-dev-abc"),);

        // Test version equality
        assert_eq!(parse("2021.1"), parse("2021.1"));
        assert_eq!(parse("2021.1-beta1"), parse("2021.1-beta1"));
        assert_eq!(parse("2021.1-alpha7"), parse("2021.1-alpha7"));
        assert_eq!(parse("2021.1-dev-abc123"), parse("2021.1-dev-abc123"));
        assert_ne!(parse("2021.1-dev-abc123"), parse("2021.1-dev-def123"));
    }

    #[test]
    fn test_version_ordering_and_equality_dev() {
        let v1 = parse("2021.3-dev-abc");
        let v2 = parse("2021.3-dev-def");

        // Exactly the same version are equal, but has no ordering
        assert_eq!(v1, v1);
        assert!(v1.partial_cmp(&v1).is_none());

        // Equal down to the dev suffix are not equal, and has no ordering
        assert_ne!(v1, v2);
        assert!(v1.partial_cmp(&v2).is_none());
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("2021.34"),
            Version {
                year: 2021,
                incremental: 34,
                pre_stable: None,
                dev: None,
            }
        );
    }

    #[test]
    fn test_parse_with_alpha() {
        assert_eq!(
            parse("2023.1-alpha77"),
            Version {
                year: 2023,
                incremental: 1,
                pre_stable: Some(PreStableType::Alpha(77)),
                dev: None,
            }
        );

        assert_eq!(
            parse("2021.34-alpha777"),
            Version {
                year: 2021,
                incremental: 34,
                pre_stable: Some(PreStableType::Alpha(777)),
                dev: None,
            }
        );
    }

    #[test]
    fn test_parse_with_beta() {
        assert_eq!(
            parse("2021.34-beta5"),
            Version {
                year: 2021,
                incremental: 34,
                pre_stable: Some(PreStableType::Beta(5)),
                dev: None,
            }
        );

        assert_eq!(
            parse("2021.34-beta453"),
            Version {
                year: 2021,
                incremental: 34,
                pre_stable: Some(PreStableType::Beta(453)),
                dev: None,
            }
        );
    }

    #[test]
    fn test_parse_with_dev() {
        assert_eq!(
            parse("2021.34-dev-0b60e4d87"),
            Version {
                year: 2021,
                incremental: 34,
                pre_stable: None,
                dev: Some("0b60e4d87".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_both_beta_and_dev() {
        assert_eq!(
            parse("2024.8-beta1-dev-e5483d"),
            Version {
                year: 2024,
                incremental: 8,
                pre_stable: Some(PreStableType::Beta(1)),
                dev: Some("e5483d".to_string()),
            }
        );
    }

    #[test]
    fn test_returns_error_on_invalid_version() {
        assert!("2021".parse::<Version>().is_err());
        assert!("not-a-version".parse::<Version>().is_err());
        assert!("".parse::<Version>().is_err());
    }

    #[test]
    fn test_returns_error_on_invalid_incremental() {
        assert!("2021.2a".parse::<Version>().is_err());
    }

    #[test]
    fn test_returns_error_on_invalid_version_type() {
        assert!("2021.2-omega".parse::<Version>().is_err());
    }

    #[test]
    fn test_returns_error_on_invalid_version_type_number() {
        assert!("2021.1-beta001".parse::<Version>().is_err());
    }

    #[test]
    fn test_returns_error_on_alpha_and_beta_in_same_version() {
        assert!("2021.1-beta5-alpha2".parse::<Version>().is_err());
    }

    #[test]
    fn test_returns_error_on_dev_without_commit_hash() {
        assert!("2021.1-dev".parse::<Version>().is_err())
    }

    #[test]
    fn test_version_display() {
        let assert_same_display = |version: &str| {
            let parsed = Version::from_str(version).unwrap();
            assert_eq!(parsed.to_string(), version);
        };

        assert_same_display("2024.8-beta1-dev-e5483d");
        assert_same_display("2024.8-beta1");
        assert_same_display("2024.8-alpha77-dev-85483d");
        assert_same_display("2024.12");
        assert_same_display("2045.2-dev-123");
    }
}
