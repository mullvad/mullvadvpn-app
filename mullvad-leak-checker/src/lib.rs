use std::{fmt, net::IpAddr};

#[cfg(feature = "am-i-mullvad")]
pub mod am_i_mullvad;
pub mod traceroute;
mod util;

#[derive(Clone, Debug)]
pub enum LeakStatus {
    NoLeak,
    LeakDetected(LeakInfo),
}

/// Details about how a leak happened
#[derive(Clone, Debug)]
pub enum LeakInfo {
    /// Managed to reach another network node on the physical interface, bypassing firewall rules.
    NodeReachableOnInterface {
        reachable_nodes: Vec<IpAddr>,
        interface: Interface,
    },

    /// Queried a <https://am.i.mullvad.net>, and was not mullvad.
    #[cfg(feature = "am-i-mullvad")]
    AmIMullvad { ip: IpAddr },
}

#[derive(Clone)]
pub enum Interface {
    Name(String),

    #[cfg(target_os = "windows")]
    Luid(windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH),

    #[cfg(target_os = "macos")]
    Index(std::num::NonZeroU32),
}

impl From<String> for Interface {
    fn from(name: String) -> Self {
        Interface::Name(name)
    }
}

impl fmt::Debug for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(arg0) => f.debug_tuple("Name").field(arg0).finish(),

            #[cfg(target_os = "windows")]
            Self::Luid(arg0) => f
                .debug_tuple("Luid")
                .field(
                    // SAFETY: u64 is valid for all bit patterns, so reading the union as a u64 is safe.
                    &unsafe { arg0.Value },
                )
                .finish(),

            #[cfg(target_os = "macos")]
            Self::Index(arg0) => f.debug_tuple("Luid").field(arg0).finish(),
        }
    }
}
