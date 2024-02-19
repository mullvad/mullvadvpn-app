//! Constrain yourself.

mod constraint;
mod set;

// Re-export bits & pieces from `constraints.rs` as needed
pub use constraint::Constraint;
pub use set::{Intersection, Set};

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

pub trait Match<T> {
    fn matches(&self, other: &T) -> bool;
}

#[cfg(test)]
pub mod test {
    pub use super::constraint::test::{any, constraint, only};
}
