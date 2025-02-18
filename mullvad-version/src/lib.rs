use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use regex_lite::Regex;

/// The Mullvad VPN app product version
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl Version {
    pub fn is_stable(&self) -> bool {
        self.pre_stable.is_none()
    }

    pub fn is_alpha(&self) -> bool {
        self.alpha().is_some()
    }

    pub fn is_beta(&self) -> bool {
        self.beta().is_some()
    }

    pub fn is_dev(&self) -> bool {
        self.dev.is_some()
    }

    pub fn alpha(&self) -> Option<u32> {
        match self.pre_stable {
            Some(PreStableType::Alpha(v)) => Some(v),
            _ => None,
        }
    }

    pub fn beta(&self) -> Option<u32> {
        match self.pre_stable {
            Some(PreStableType::Beta(v)) => Some(v),
            _ => None,
        }
    }

    /// Returns the ordering between two versions. The reason the Ord trait isn't implemented
    /// is that dev versions are not ordered, so for example 2025.1-dev-fd42ad is equal to
    /// 2025.1-dev-af23b4 when it comes to the version ordering, but they are not equal when it
    /// comes to the derived Eq and PartialEq implementations.
    ///
    /// The following order is used: stable > beta > alpha
    /// If two versions are identical except for one being a dev version
    /// (ending with the suffix -dev-[SHA]), the dev version is considered greater than
    /// the non-dev version, but if both are dev versions they are considered equal
    /// since the dev version SHA is not ordered.
    pub fn version_ordering(&self, other: &Self) -> Ordering {
        let type_ordering = if self.is_stable() || other.is_stable() {
            match (self.is_stable(), other.is_stable()) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (_, _) => Ordering::Equal,
            }
        } else if self.is_beta() || other.is_beta() {
            self.beta().cmp(&other.beta())
        } else if self.is_alpha() || other.is_alpha() {
            self.alpha().cmp(&other.alpha())
        } else {
            Ordering::Equal
        };

        // The dev vs non-dev ordering. For a version of a given type, if all else is equal
        // a dev version is greater than a non-dev version.
        let dev_ordering = match (self.is_dev(), other.is_dev()) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (_, _) => Ordering::Equal,
        };

        self.year
            .cmp(&other.year)
            .then(self.incremental.cmp(&other.incremental))
            .then(type_ordering)
            .then(dev_ordering)
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

    #[test]
    fn test_version_ordering() {
        // Test year comparison
        assert_eq!(
            parse("2022.1").version_ordering(&parse("2021.1")),
            Ordering::Greater
        );

        // Test incremental comparison
        assert_eq!(
            parse("2021.2").version_ordering(&parse("2021.1")),
            Ordering::Greater
        );

        // Test stable vs pre-release
        assert_eq!(
            parse("2021.1").version_ordering(&parse("2021.1-beta1")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.1").version_ordering(&parse("2021.1-alpha1")),
            Ordering::Greater
        );

        // Test beta vs alpha
        assert_eq!(
            parse("2021.1-beta1").version_ordering(&parse("2021.1-alpha1")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.1-beta1").version_ordering(&parse("2021.1-alpha2")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.2-alpha1").version_ordering(&parse("2021.1-beta2")),
            Ordering::Greater
        );

        // Test version numbers within same type
        assert_eq!(
            parse("2021.1-beta2").version_ordering(&parse("2021.1-beta1")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.1-alpha2").version_ordering(&parse("2021.1-alpha1")),
            Ordering::Greater
        );

        // Test dev versions
        assert_eq!(
            parse("2021.1-dev-abc").version_ordering(&parse("2021.1")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.2").version_ordering(&parse("2021.1-dev-abc")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.1-dev-abc").version_ordering(&parse("2021.1-beta1")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2021.1-dev-abc").version_ordering(&parse("2021.1-alpha1")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2025.1-dev-abc").version_ordering(&parse("2025.1-beta1-dev-abc")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2025.1-dev-abc").version_ordering(&parse("2025.1-beta2-dev-abc")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2025.1-dev-abc").version_ordering(&parse("2025.1-alpha2-dev-abc")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2025.1-beta1-dev-abc").version_ordering(&parse("2025.1-alpha7-dev-abc")),
            Ordering::Greater
        );
        assert_eq!(
            parse("2025.2-alpha1-dev-abc").version_ordering(&parse("2025.1-beta7-dev-abc")),
            Ordering::Greater
        );

        // Test equal versions
        assert_eq!(parse("2021.1"), parse("2021.1"));
        assert_eq!(parse("2021.1-beta1"), parse("2021.1-beta1"));
        assert_eq!(parse("2021.1-dev-abc123"), parse("2021.1-dev-abc123"));
        assert_ne!(parse("2021.1-dev-abc123"), parse("2021.1-dev-def123"));

        assert_eq!(
            parse("2021.1").version_ordering(&parse("2021.1")),
            Ordering::Equal
        );
        assert_eq!(
            parse("2021.1-beta1").version_ordering(&parse("2021.1-beta1")),
            Ordering::Equal
        );
        assert_eq!(
            parse("2021.1-dev-abc123").version_ordering(&parse("2021.1-dev-abc123")),
            Ordering::Equal
        );
        assert_eq!(
            parse("2021.1-dev-abc123").version_ordering(&parse("2021.1-dev-abc123")),
            Ordering::Equal
        );
    }

    #[test]
    fn test_version_regex() {
        assert!(VERSION_REGEX.is_match("2020.4"));
        assert!(VERSION_REGEX.is_match("2020.4-beta3"));
        assert!(VERSION_REGEX.is_match("2020.4-alpha3"));
        assert!(VERSION_REGEX.is_match("2020.4-beta3"));
        assert!(VERSION_REGEX.is_match("2020.4-alpha99"));
        assert!(VERSION_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5-alpha2-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.4"));
        assert!(VERSION_REGEX.is_match("2020.5-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5-beta1-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5-alpha4-dev-f16be4"));
        assert!(VERSION_REGEX.is_match("2020.5"));
        assert!(VERSION_REGEX.is_match("2020.5-beta1"));
        assert!(VERSION_REGEX.is_match("2020.5-alpha1"));
    }

    #[test]
    fn test_parse() {
        let version = "2021.34";
        let parsed = parse(version);
        assert_eq!(parsed.year, 2021);
        assert_eq!(parsed.incremental, 34);
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev, None);
        assert!(parsed.is_stable());
    }

    #[test]
    fn test_parse_with_alpha() {
        let version = "2023.1-alpha77";
        let parsed = parse(version);
        assert_eq!(parsed.year, 2023);
        assert_eq!(parsed.incremental, 1);
        assert_eq!(parsed.alpha(), Some(77));
        assert_eq!(parsed.beta(), None);
        assert_eq!(parsed.dev, None);
        assert!(!parsed.is_stable());

        let version = "2021.34-alpha777";
        let parsed = parse(version);
        assert_eq!(parsed.alpha(), Some(777));
    }

    #[test]
    fn test_parse_with_beta() {
        let version = "2021.34-beta5";
        let parsed = parse(version);
        assert_eq!(parsed.year, 2021);
        assert_eq!(parsed.incremental, 34);
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), Some(5));
        assert_eq!(parsed.dev, None);
        assert!(!parsed.is_stable());

        let version = "2021.34-beta453";
        let parsed = parse(version);
        assert_eq!(parsed.beta(), Some(453));
    }

    #[test]
    fn test_parse_with_dev() {
        let version = "2021.34-dev-0b60e4d87";
        let parsed = parse(version);
        assert_eq!(parsed.year, 2021);
        assert_eq!(parsed.incremental, 34);
        assert!(parsed.is_stable());
        assert_eq!(parsed.dev, Some("0b60e4d87".to_string()));
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), None);
    }

    #[test]
    fn test_parse_both_beta_and_dev() {
        let version = "2024.8-beta1-dev-e5483d";
        let parsed = parse(version);
        assert_eq!(parsed.year, 2024);
        assert_eq!(parsed.incremental, 8);
        assert_eq!(parsed.alpha(), None);
        assert_eq!(parsed.beta(), Some(1));
        assert_eq!(parsed.dev, Some("e5483d".to_string()));
        assert!(!parsed.is_stable());
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

    fn parse(version: &str) -> Version {
        version.parse().unwrap()
    }

    #[test]
    fn test_version_display() {
        let version = "2024.8-beta1-dev-e5483d";
        let parsed: Version = version.parse().unwrap();

        assert_eq!(format!("{parsed}"), version);

        let version = "2024.8-beta1";
        let parsed: Version = version.parse().unwrap();

        assert_eq!(format!("{parsed}"), version);

        let version = "2024.8-alpha77-dev-85483d";
        let parsed: Version = version.parse().unwrap();

        assert_eq!(format!("{parsed}"), version);

        let version = "2024.12";
        let parsed: Version = version.parse().unwrap();

        assert_eq!(format!("{parsed}"), version);
    }
}
