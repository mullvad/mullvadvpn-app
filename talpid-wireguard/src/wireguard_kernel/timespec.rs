use std::time::{Duration, SystemTime};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// `__kernel_timespec` from uapi/linux/time_types.h.
///
/// This time type is used for the WireGuard last handshake timestamp.
/// Source: Linux kernel source code
/// - drivers/net/wireguard/netlink.c
/// - include/uapi/linux/time_types.h
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub struct KernelTimespec {
    /// seconds
    pub(crate) tv_sec: libc::c_longlong,
    /// nanoseconds
    pub(crate) tv_nsec: libc::c_longlong,
}

impl KernelTimespec {
    pub fn as_systemtime(&self) -> Option<SystemTime> {
        let (tv_sec, tv_nsec) = (
            Duration::from_secs(self.tv_sec.try_into().ok()?),
            Duration::from_nanos(
                self
                .tv_nsec
                .try_into()
                // Source: man timespec
                .expect("tv_nsec is at most 999_999_999"),
            ),
        );
        // handshake_{sec,nsec} are relative to UNIX_EPOCH
        // https://www.wireguard.com/xplatform/
        Some(SystemTime::UNIX_EPOCH + tv_sec + tv_nsec)
    }
}
