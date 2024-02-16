//! A limited variant of Sets.

pub trait Set<T> {
    fn is_subset(&self, other: &T) -> bool;
}
