/// A 1-1 mapping of [std::net::Ipv4Addr] memorywise, but uniffi compatible.
/// Allows for easy conversion between the [std] type and this one.
#[derive(uniffi::Record, Copy, Clone, PartialEq, Eq)]
pub struct Ipv4Addr {
    bits: u32,
}
#[uniffi::export]
impl Ipv4Addr {
    pub fn as_string(&self) -> String {
        std::net::Ipv4Addr::from(*self).to_string()
    }
}

impl std::fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&std::net::Ipv4Addr::from(*self), f)
    }
}
impl std::fmt::Debug for Ipv4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&std::net::Ipv4Addr::from(*self), f)
    }
}
impl From<std::net::Ipv4Addr> for Ipv4Addr {
    fn from(value: std::net::Ipv4Addr) -> Self {
        Self {
            bits: value.to_bits(),
        }
    }
}
impl From<Ipv4Addr> for std::net::Ipv4Addr {
    fn from(val: Ipv4Addr) -> Self {
        Self::from_bits(val.bits)
    }
}

const fn a_to_b(val: u128) -> (u64, u64) {
    ((val >> 64) as u64, val as u64)
}
const fn b_to_a((a, b): (u64, u64)) -> u128 {
    ((a as u128) << 64) + (b as u128)
}

/// A 1-1 mapping of [std::net::Ipv6Addr] memorywise, but uniffi compatible.
/// Allows for easy conversion between the [std] type and this one.
#[derive(uniffi::Record, Copy, Clone, PartialEq, Eq)]
pub struct Ipv6Addr {
    bits1: u64,
    bits2: u64,
}
#[uniffi::export]
impl Ipv6Addr {
    pub fn as_string(&self) -> String {
        std::net::Ipv6Addr::from(*self).to_string()
    }
}
impl std::fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&std::net::Ipv6Addr::from(*self), f)
    }
}
impl std::fmt::Debug for Ipv6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&std::net::Ipv6Addr::from(*self), f)
    }
}
impl From<std::net::Ipv6Addr> for Ipv6Addr {
    fn from(value: std::net::Ipv6Addr) -> Self {
        let (bits1, bits2) = a_to_b(value.to_bits());
        Self { bits1, bits2 }
    }
}
impl From<Ipv6Addr> for std::net::Ipv6Addr {
    fn from(val: Ipv6Addr) -> Self {
        let Ipv6Addr { bits1, bits2 } = val;
        Self::from_bits(b_to_a((bits1, bits2)))
    }
}
const _: () = {
    // this is large enough where both bit sets are not 0
    const NUM: u128 = const { u64::MAX as u128 + 10 };
    ["a = f (g a)"][(b_to_a(a_to_b(NUM)) - NUM) as usize];
};
