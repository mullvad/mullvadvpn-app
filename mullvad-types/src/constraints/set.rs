//! A limited variant of Sets.

pub trait Set<T> {
    fn is_subset(&self, other: &T) -> bool;
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
