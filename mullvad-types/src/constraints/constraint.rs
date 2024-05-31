//! General constraints.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::Intersection;

/// Limits the set of [`crate::relay_list::Relay`]s that a `RelaySelector` may select.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Constraint<T> {
    Any,
    Only(T),
}

impl<T: Intersection> Intersection for Constraint<T> {
    /// Define the intersection between two arbitrary [`Constraint`]s.
    ///
    /// This operation may be compared to the set operation with the same name.
    /// In contrast to the general set intersection, this function represents a
    /// very specific case where [`Constraint::Any`] is equivalent to the set
    /// universe and [`Constraint::Only`] represents a singleton set. Notable is
    /// that the representation of any empty set is [`Option::None`].
    fn intersection(self, other: Constraint<T>) -> Option<Constraint<T>> {
        use Constraint::*;
        match (self, other) {
            (Any, Any) => Some(Any),
            (Only(t), Any) | (Any, Only(t)) => Some(Only(t)),
            // Pick any of `left` or `right` if they are the same.
            (Only(left), Only(right)) => left.intersection(right).map(Only),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Constraint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Constraint::Any => "any".fmt(f),
            Constraint::Only(value) => fmt::Display::fmt(value, f),
        }
    }
}

impl<T> Constraint<T> {
    pub fn unwrap(self) -> T {
        match self {
            Constraint::Any => panic!("called `Constraint::unwrap()` on an `Any` value"),
            Constraint::Only(value) => value,
        }
    }

    pub fn unwrap_or(self, other: T) -> T {
        match self {
            Constraint::Any => other,
            Constraint::Only(value) => value,
        }
    }

    pub fn or(self, other: Constraint<T>) -> Constraint<T> {
        match self {
            Constraint::Any => other,
            Constraint::Only(value) => Constraint::Only(value),
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Constraint<U> {
        match self {
            Constraint::Any => Constraint::Any,
            Constraint::Only(value) => Constraint::Only(f(value)),
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> Constraint<U>>(self, f: F) -> Constraint<U> {
        match self {
            Constraint::Any => Constraint::Any,
            Constraint::Only(value) => f(value),
        }
    }

    pub const fn is_any(&self) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Only(_value) => false,
        }
    }

    pub const fn is_only(&self) -> bool {
        !self.is_any()
    }

    pub const fn as_ref(&self) -> Constraint<&T> {
        match self {
            Constraint::Any => Constraint::Any,
            Constraint::Only(ref value) => Constraint::Only(value),
        }
    }

    pub fn option(self) -> Option<T> {
        match self {
            Constraint::Any => None,
            Constraint::Only(value) => Some(value),
        }
    }

    /// Returns true if the constraint is an `Only` and the value inside of it matches a predicate.
    pub fn is_only_and(self, f: impl FnOnce(T) -> bool) -> bool {
        self.option().is_some_and(f)
    }
}

impl<T: PartialEq> Constraint<T> {
    pub fn matches_eq(&self, other: &T) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value == other,
        }
    }
}

// Using the default attribute fails on Android
#[allow(clippy::derivable_impls)]
impl<T> Default for Constraint<T> {
    fn default() -> Self {
        Constraint::Any
    }
}

impl<T> From<Option<T>> for Constraint<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Constraint::Only(value),
            None => Constraint::Any,
        }
    }
}

impl<T> From<T> for Constraint<T> {
    fn from(value: T) -> Self {
        Constraint::Only(value)
    }
}

impl<T: Copy> Copy for Constraint<T> {}

impl<T: fmt::Debug + Clone + FromStr> FromStr for Constraint<T> {
    type Err = T::Err;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("any") {
            return Ok(Self::Any);
        }
        Ok(Self::Only(T::from_str(value)?))
    }
}

// Clap

#[cfg(feature = "clap")]
#[derive(fmt::Debug, Clone)]
pub struct ConstraintParser<T>(T);

#[cfg(feature = "clap")]
impl<T: fmt::Debug + Clone + clap::builder::ValueParserFactory> clap::builder::ValueParserFactory
    for Constraint<T>
where
    <T as clap::builder::ValueParserFactory>::Parser: Sync + Send + Clone,
{
    type Parser = ConstraintParser<T::Parser>;

    fn value_parser() -> Self::Parser {
        ConstraintParser(T::value_parser())
    }
}

#[cfg(feature = "clap")]
impl<T: clap::builder::TypedValueParser> clap::builder::TypedValueParser for ConstraintParser<T>
where
    T::Value: fmt::Debug,
{
    type Value = Constraint<T::Value>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        if value.eq_ignore_ascii_case("any") {
            return Ok(Constraint::Any);
        }
        self.0.parse_ref(cmd, arg, value).map(Constraint::Only)
    }
}
