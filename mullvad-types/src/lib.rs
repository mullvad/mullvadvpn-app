pub mod access_method;
pub mod account;
pub mod auth_failed;
pub mod constraints;
pub mod custom_list;
pub mod device;
pub mod endpoint;
pub mod location;
pub mod relay_constraints;
pub mod relay_list;
pub mod settings;
pub mod states;
pub mod version;
pub mod wireguard;

mod custom_tunnel;
pub use crate::custom_tunnel::*;

// b"mole" is [ 0x6d, 0x6f 0x6c, 0x65 ]
#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x6d6f6c65;
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x6d6f6c65;

/// Any type that wish to implement `Intersection` should make sure that the
/// following properties are upheld:
///
/// - idempotency (if there is an identity element)
/// - commutativity
/// - associativity
pub trait Intersection: Sized {
    fn intersection(self, other: Self) -> Option<Self>;
}

pub use intersection_derive::Intersection;

macro_rules! impl_intersection_partialeq {
    ($ty:ty) => {
        impl $crate::Intersection for $ty {
            fn intersection(self, other: Self) -> Option<Self> {
                if self == other {
                    Some(self)
                } else {
                    None
                }
            }
        }
    };
}

impl_intersection_partialeq!(u16);
impl_intersection_partialeq!(bool);

// NOTE: this implementation does not do what you may expect of an intersection
impl_intersection_partialeq!(relay_constraints::Providers);
// NOTE: should take actual intersection
impl_intersection_partialeq!(relay_constraints::LocationConstraint);
impl_intersection_partialeq!(relay_constraints::Ownership);
// NOTE: it contains an inner constraint
impl_intersection_partialeq!(relay_constraints::TransportPort);
impl_intersection_partialeq!(talpid_types::net::TransportProtocol);
impl_intersection_partialeq!(talpid_types::net::TunnelType);
impl_intersection_partialeq!(talpid_types::net::IpVersion);
