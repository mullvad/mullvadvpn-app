//! Constrain yourself.

mod constraint;

// Re-export bits & pieces from `constraints.rs` as needed
pub use constraint::Constraint;

use crate::relay_constraints;

pub trait Match<T> {
    fn matches(&self, other: &T) -> bool;
}
impl<T: Match<U>, U> Match<U> for Constraint<T> {
    fn matches(&self, other: &U) -> bool {
        match *self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value.matches(other),
        }
    }
}

#[macro_export]
macro_rules! impl_intersection_partialeq {
    ($ty:ty) => {
        impl $crate::Intersection for $ty {
            fn intersection(self, other: Self) -> Option<Self> {
                if self == other { Some(self) } else { None }
            }
        }
    };
}

// NOTE: This docstring cannot link to `mullvad_relay_selector::relay_selector::query` and
// `RETRY_ORDER`as `mullvad_relay_selector` is not a dependency of this crate
/// The intersection of two sets of criteria on [`Relay`](crate::relay_list::Relay)s is another
/// criteria which matches the given relay iff both of the original criteria matched. It is
/// primarily used by the relay selector to check whether a given connection method is compatible
/// with the users settings.
///
/// # Examples
///
/// The [`Intersection`] implementation of
/// `mullvad_relay_selector::relay_selector::query::RelayQuery` upholds the following properties:
///
/// * If two `RelayQuery`s differ such that no relay matches both, [`Option::None`] is returned:
/// ```rust, ignore
/// # use mullvad_relay_selector::query::builder::RelayQueryBuilder;
/// # use crate::relay_constraints::Ownership;
/// let query_a = RelayQueryBuilder::new().ownership(Ownership::MullvadOwned).build();
/// let query_b = RelayQueryBuilder::new().ownership(Ownership::Rented).build();
/// assert_eq!(query_a.intersection(query_b), None);
/// ```
///
/// * Otherwise, a new `RelayQuery` is returned where each constraint is as specific as possible.
///   See [`Constraint`] for further details.
/// ```rust, ignore
/// let query_a = RelayQueryBuilder::new().build();
/// let query_b = RelayQueryBuilder::new().location(city("Sweden", "Gothenburg")).build();
///
/// let result = relay_selector.get_relay_by_query(query_a.intersection(query_b).unwrap());
/// assert!(result.is_ok());
/// ```
///
/// This way, if the mullvad app wants to check if the user's relay settings
/// are compatible with any other `RelayQuery`, for examples those defined by
/// `RETRY_ORDER` , taking the intersection between them will never result in
/// a situation where the app can override the user's preferences.
///
///
/// The macro recursively applies the intersection on each field of the struct and returns the
/// resulting type or `None` if any of the intersections failed to overlap.
///
/// The macro requires the types of each field to also implement [`Intersection`], which may be done
/// using this derive macro, the
///
/// # Implementing [`Intersection`]
///
/// For structs where each field already implements `Intersection`, the easiest way to implement the
/// trait is using the derive macro. Using the derive macro on `RelayQuery`
/// ```rust, ignore
/// #[derive(Intersection)]
/// pub struct RelayQuery {
///     pub location: Constraint<LocationConstraint>,
///     pub providers: Constraint<Providers>,
///     pub ownership: Constraint<Ownership>,
///     pub wireguard_constraints: WireguardRelayQuery,
/// }
/// ```
///
/// produces an implementation like this:
///
/// ```rust, ignore
/// impl Intersection for RelayQuery {
///     fn intersection(self, other: Self) -> Option<Self>
///     where
///         Self: PartialEq,
///         Self: Sized,
///     {
///         Some(RelayQuery {
///             location: self.location.intersection(other.location)?,
///             providers: self.providers.intersection(other.providers)?,
///             ownership: self.ownership.intersection(other.ownership)?,
///             wireguard_constraints: self
///                 .wireguard_constraints
///                 .intersection(other.wireguard_constraints)?,
///         })
///     }
/// }
/// ```
///
/// For types that cannot "overlap", e.g. they only intersect if they are equal, the declarative
/// macro [`impl_intersection_partialeq`] can be used.
///
/// For less trivial cases, the trait needs to be implemented manually. When doing so, make sure
/// that the following properties are upheld:
///
/// - idempotence (if there is an identity element)
/// - commutativity
/// - associativity
pub trait Intersection: Sized {
    fn intersection(self, other: Self) -> Option<Self>;
}

// Note that deriving `Intersection` for using `impl_intersection_partialeq`
// may not do what you expect for data structures that represent/wrap sets, such as
// `Vec<T>` or `HashSet<T>`. `Constraint::Only` will only match if the
// `Vec<T>` or `HashSet<T>` is exactly the same, not if they contain overlapping elements.
impl_intersection_partialeq!(u16);
impl_intersection_partialeq!(bool);
impl_intersection_partialeq!(relay_constraints::Providers);
impl_intersection_partialeq!(relay_constraints::LocationConstraint);
impl_intersection_partialeq!(relay_constraints::Ownership);
impl_intersection_partialeq!(talpid_types::net::TransportProtocol);
impl_intersection_partialeq!(talpid_types::net::IpVersion);
impl_intersection_partialeq!(relay_constraints::AllowedIps);

#[cfg(test)]
mod tests {
    use crate::Intersection;

    // ── Struct derive ─────────────────────────────────────────────────────────

    /// A simple struct where every field implements `Intersection` via `PartialEq`.
    #[derive(Debug, PartialEq, Intersection)]
    struct Point {
        x: u16,
        y: u16,
    }

    #[test]
    fn struct_same_fields_intersect() {
        let a = Point { x: 1, y: 2 };
        let b = Point { x: 1, y: 2 };
        assert_eq!(a.intersection(b), Some(Point { x: 1, y: 2 }));
    }

    #[test]
    fn struct_differing_field_yields_none() {
        let a = Point { x: 1, y: 2 };
        let b = Point { x: 1, y: 99 };
        assert_eq!(a.intersection(b), None);
    }

    // ── Enum derive ───────────────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Intersection)]
    enum Color {
        /// Unit variant — no inner data.
        Red,
        Green,
        /// Newtype variant — inner value must itself implement `Intersection`.
        Custom(u16),
    }

    /// Same unit variant × same unit variant → `Some`.
    #[test]
    fn enum_unit_same_variant_intersects() {
        assert_eq!(Color::Red.intersection(Color::Red), Some(Color::Red));
        assert_eq!(Color::Green.intersection(Color::Green), Some(Color::Green));
    }

    /// Different variants → `None`.
    #[test]
    fn enum_different_variants_yield_none() {
        assert_eq!(Color::Red.intersection(Color::Green), None);
        assert_eq!(Color::Green.intersection(Color::Custom(1)), None);
    }

    /// Same newtype variant, same inner value → `Some`.
    #[test]
    fn enum_newtype_same_value_intersects() {
        assert_eq!(
            Color::Custom(42).intersection(Color::Custom(42)),
            Some(Color::Custom(42))
        );
    }

    /// Same newtype variant, differing inner value → `None`.
    #[test]
    fn enum_newtype_different_value_yields_none() {
        assert_eq!(Color::Custom(1).intersection(Color::Custom(2)), None);
    }

    /// Intersection is commutative for enums.
    #[test]
    fn enum_intersection_is_commutative() {
        assert_eq!(
            Color::Red.clone().intersection(Color::Green.clone()),
            Color::Green.intersection(Color::Red),
        );
        assert_eq!(
            Color::Custom(5).intersection(Color::Custom(5)),
            Color::Custom(5).intersection(Color::Custom(5)),
        );
    }
}
