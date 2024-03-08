bitflags::bitflags! {
    /// All enum values of address flags can be iterated via `flag <<= 1`, starting from 1.
    /// See https://www.manpagez.com/man/4/route/.
    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
    pub struct AddressFlag: i32 {
        /// Destination socket address
        const RTA_DST       = libc::RTA_DST;
        /// Gateway socket address
        const RTA_GATEWAY   = libc::RTA_GATEWAY;
        /// Netmask socket address
        const RTA_NETMASK   = libc::RTA_NETMASK;
        /// Cloning mask socket address
        const RTA_GENMASK   = libc::RTA_GENMASK;
        /// Interface name socket address
        const RTA_IFP       = libc::RTA_IFP;
        /// Interface address socket address
        const RTA_IFA       = libc::RTA_IFA;
        /// Socket address for author of redirect
        const RTA_AUTHOR    = libc::RTA_AUTHOR;
        /// Socket address for `NEWADDR`, broadcast or point-to-point destination address
        const RTA_BRD       = libc::RTA_BRD;
    }
}
