//! Abstractions for working with app version rollout threshold.
use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::ops::RangeInclusive;
use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize, de::Error};

/// Rollout threshold. Any version in the response below this threshold will be ignored
///
/// INVARIANT: The inner f32 must be in the `VALID_ROLLOUT` range.
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Rollout(f32);

/// Accept *any* version (rollout >= 0) when querying for app info.
pub const IGNORE: Rollout = Rollout(0.);

/// Accept any version (rollout > 0) when querying for app info.
/// Only versions with a non-zero rollout are supported.
pub const SUPPORTED_VERSION: Rollout = Rollout(f32::EPSILON);

/// Accept only fully rolled out versions (rollout >= 1) when querying for app info.
pub const FULLY_ROLLED_OUT: Rollout = Rollout(1.);

const VALID_ROLLOUT: RangeInclusive<f32> = 0.0..=1.0;

impl Rollout {
    /// Calculate the threshold used to determine if a client is included in the current rollout of
    /// some release.
    ///
    /// Invariant: 0.0 < threshold <= 1.0
    ///
    /// 0.0 is a special-cased rollout value reserved for complete rollbacks. See [IGNORE].
    pub fn threshold(rollout_threshold_seed: u32, version: mullvad_version::Version) -> Self {
        use rand::{Rng, SeedableRng, rngs::SmallRng};
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(rollout_threshold_seed.to_string());
        hasher.update(version.to_string());
        let hash = hasher.finalize();
        let seed: &[u8; 32] = hash.first_chunk().expect("SHA256 hash is 32 bytes");
        let mut rng = SmallRng::from_seed(*seed);
        let threshold = rng.random_range(SUPPORTED_VERSION.0..=FULLY_ROLLED_OUT.0);
        Self::try_from(threshold).expect("threshold is within the Rollout domain")
    }

    /// Generate a special seed used to calculate at which rollout percentage a client should be
    /// notified about a new release.
    ///
    /// See [Rollout::threshold] for details.
    pub fn seed() -> u32 {
        rand::random()
    }

    /// A full rollout includes all users
    pub const fn complete() -> Self {
        FULLY_ROLLED_OUT
    }
}

pub fn is_complete_rollout(b: impl std::borrow::Borrow<Rollout>) -> bool {
    // TODO: do we actually need this? if so, should we bake it into Rollout::eq?
    //(b.borrow() - complete_rollout()).abs() < f32::EPSILON
    b.borrow() == &FULLY_ROLLED_OUT
}

impl TryFrom<f32> for Rollout {
    type Error = anyhow::Error;

    fn try_from(rollout: f32) -> Result<Self, Self::Error> {
        if !rollout.is_finite() {
            bail!("rollout value must be a finite number, but was {rollout}");
        }

        if !VALID_ROLLOUT.contains(&rollout) {
            bail!(
                "rollout value {rollout} is outside valid range {}..={}",
                VALID_ROLLOUT.start(),
                VALID_ROLLOUT.end(),
            );
        }

        Ok(Rollout(rollout))
    }
}

impl Eq for Rollout {}

#[expect(clippy::derive_ord_xor_partial_ord)] // we impl Ord in terms of PartalOrd, so it's fine
impl Ord for Rollout {
    fn cmp(&self, other: &Self) -> Ordering {
        debug_assert!(self.0.is_finite());
        debug_assert!(other.0.is_finite());
        self.partial_cmp(other).expect("rollout is always in 0..=1")
    }
}

impl Display for Rollout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Rollout {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rollout: f32 = s.parse()?;
        Rollout::try_from(rollout)
    }
}

impl<'de> Deserialize<'de> for Rollout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let rollout = f32::deserialize(deserializer)?;

        Rollout::try_from(rollout)
            .map_err(|e| e.to_string())
            .map_err(D::Error::custom)
    }
}

impl Serialize for Rollout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "arbitrary")]
/// Generators for [Rollout].
pub mod arbitrary {
    use super::*;

    use proptest::prelude::*;

    /// Generate *any* arbitrary [Rollout] values.
    ///
    /// This generator assume that [VALID_ROLLOUT] represent all valid rollouts.
    #[expect(dead_code)]
    pub fn arb_any_rollout() -> impl Strategy<Value = Rollout> {
        VALID_ROLLOUT.prop_map(Rollout)
    }

    /// Generate an arbitrary [Rollout] values.
    ///
    /// This generator is heavily biased towards edge cases such as zero rollout, full rollout etc.
    #[expect(dead_code)]
    pub fn arb_rollout() -> impl Strategy<Value = Rollout> {
        let any = arb_any_rollout();
        let edge_cases = prop_oneof![
            Just(IGNORE),            // = 0
            Just(SUPPORTED_VERSION), // > 0
            Just(FULLY_ROLLED_OUT)   // = 1
        ];
        // Let's say that any of the edge-cases should be generated with a 1/5 probabilty.
        prop_oneof![
            80 => any,
            20 => edge_cases
        ]
    }
}

#[cfg(test)]
mod test {
    use super::arbitrary::*;
    use super::*;

    use insta::{assert_snapshot, assert_yaml_snapshot};
    use proptest::prelude::*;

    proptest! {
         /// Assert that all rollout values from 0 up to 1 are valid rollouts.
         #[test]
         fn valid_rollout(r in arb_rollout()) {
             Rollout::from_str(&r.to_string()).unwrap();
         }

         /// Test that inequality works as expected (i.e. as for floating point numbers).
         #[test]
         fn rollout_inequality(r1 in arb_rollout(), r2 in arb_rollout()) {
            if r1.0 < r2.0 {
                assert!(r1 < r2)
            } else if r1.0 > r2.0 {
                assert!(r1 > r2)
            }
         }

         /// Test that eqaulity works as expected (i.e. as for floating point numbers).
         #[test]
         fn rollout_identity(rollout in arb_rollout()) {
             assert_eq!(rollout, rollout)
         }
    }

    const GOOD_ROLLOUT_EXAMPLES: &[f32] = &[
        -0.0,                // 0%
        0.0,                 // 0%
        -0.0 + f32::EPSILON, // > 0%
        1.0 / 3.0,           // 33%
        1.0 - f32::EPSILON,  // 99%
        1.0,                 // 100%
    ];

    const BAD_ROLLOUT_EXAMPLES: &[f32] = &[
        -f32::EPSILON,
        1.0 + f32::EPSILON,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        100.0,
    ];

    #[test]
    fn test_rollout_serialization() {
        for &valid_rollout in GOOD_ROLLOUT_EXAMPLES {
            let serialized_f32 = serde_json::to_string(&valid_rollout).unwrap();
            let deserialized_rollout: Rollout = serde_json::from_str(&serialized_f32).unwrap();
            let serialized_rollout = serde_json::to_string(&deserialized_rollout).unwrap();

            assert_eq!(deserialized_rollout.0, valid_rollout);
            assert_eq!(serialized_rollout, serialized_f32);
        }
    }

    #[test]
    fn test_rollout_deserialize_bad() {
        for &bad_rollout in BAD_ROLLOUT_EXAMPLES {
            let rollout_str = bad_rollout.to_string();
            serde_json::from_str::<Rollout>(&rollout_str)
                .expect_err("must fail to deserialize bad rollout");
        }
    }

    /// Test that the `Display` impl of [Rollout] makes sense.
    /// Note clap requires that `Display` must be the inverse of `FromStr`.
    #[test]
    fn test_rollout_display() {
        let string_reprs = GOOD_ROLLOUT_EXAMPLES
            .iter()
            .map(|&f| format!("{f} => {}\n", Rollout::try_from(f).unwrap()))
            .collect::<String>();

        assert_snapshot!(&string_reprs);
    }

    #[test]
    /// Check that the implementation of [rollout_threshold] yields different threshold values as
    /// app version number progresses.
    ///
    /// Note that there is a chance for repetition - we are effectively mapping a 256 byte hash to
    /// the fractional part of an [f32], which is a much smaller domain.
    fn test_rollout_threshold_uniqueness() {
        let seed = 4; // Chosen by fair dice roll. Guaranteed to be random.
        let v20254: mullvad_version::Version = "2025.4".parse().unwrap();
        let v20255: mullvad_version::Version = "2025.5".parse().unwrap();
        assert_ne!(
            Rollout::threshold(seed, v20254.clone()),
            Rollout::threshold(seed, v20255.clone())
        );
        assert_yaml_snapshot!(Rollout::threshold(seed, v20254));
        assert_yaml_snapshot!(Rollout::threshold(seed, v20255));
    }
}
