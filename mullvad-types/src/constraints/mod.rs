//! Constrain yourself.

mod constraint;

// Re-export bits & pieces from `constraints.rs` as needed
pub use constraint::Constraint;

/// A limited variant of Sets.
pub trait Set<T> {
    fn is_subset(&self, other: &T) -> bool;
}

pub trait Match<T> {
    fn matches(&self, other: &T) -> bool;
}

/// Any type that wish to implement `Intersection` should make sure that the
/// following properties are upheld:
///
/// - idempotency (if there is an identity element)
/// - commutativity
/// - associativity
pub trait Intersection {
    fn intersection(self, other: Self) -> Option<Self>
    where
        Self: PartialEq,
        Self: Sized;
}

impl<T: Match<U>, U> Match<U> for Constraint<T> {
    fn matches(&self, other: &U) -> bool {
        match *self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value.matches(other),
        }
    }
}

impl<T: Set<U>, U> Set<Constraint<U>> for Constraint<T> {
    fn is_subset(&self, other: &Constraint<U>) -> bool {
        match self {
            Constraint::Any => other.is_any(),
            Constraint::Only(ref constraint) => match other {
                Constraint::Only(ref other_constraint) => constraint.is_subset(other_constraint),
                _ => true,
            },
        }
    }
}

#[cfg(test)]
pub mod proptest {
    pub use super::constraint::proptest::{any, constraint, only};
}
