//! TODO

use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::ops::RangeInclusive;
use std::str::FromStr;

use anyhow::{Context, bail};
use itertools::Itertools;
use mullvad_version::PreStableType;
use serde::{Deserialize, Serialize, de::Error};

use crate::format::{self, Installer, Response};

/// Rollout threshold. Any version in the response below this threshold will be ignored
///
/// INVARIANT: The inner f32 must be in the `VALID_ROLLOUT` range.
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Rollout(pub(super) f32);
