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
    pub beta: Option<String>,
}

impl Version {
    pub fn parse(version: &str) -> Version {
        Version::from_str(version).unwrap()
    }
}

impl Display for Version {
    /// Format Version as a string: year.incremental{-beta}
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Version {
            year,
            incremental,
            beta,
        } = &self;
        match beta {
            Some(beta) => write!(f, "{year}.{incremental}-{beta}"),
            None => write!(f, "{year}.{incremental}"),
        }
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        const VERSION_REGEX: &str =
            r"^20([0-9]{2})\.([1-9][0-9]?)(-beta([1-9][0-9]?))?(-dev-[0-9a-f]+)?$";
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(VERSION_REGEX).unwrap());

        let captures = RE
            .captures(version)
            .ok_or_else(|| format!("Version does not match expected format: {version}"))?;
        let year = captures.get(1).expect("Missing year").as_str().to_owned();
        let incremental = captures
            .get(2)
            .ok_or("Missing incremental")?
            .as_str()
            .to_owned();
        let beta = captures.get(4).map(|m| m.as_str().to_owned());

        Ok(Version {
            year,
            incremental,
            beta,
        })
    }
}
