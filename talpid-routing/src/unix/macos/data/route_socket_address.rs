use super::{AddressFlag, Error, Result};
use nix::sys::socket::{SockaddrLike, SockaddrStorage};

#[derive(Debug, Clone, PartialEq)]
pub enum RouteSocketAddress {
    /// Corresponds to RTA_DST
    Destination(Option<SockaddrStorage>),
    /// RTA_GATEWAY
    Gateway(Option<SockaddrStorage>),
    /// RTA_NETMASK
    Netmask(Option<SockaddrStorage>),
    /// RTA_GENMASK
    CloningMask(Option<SockaddrStorage>),
    /// RTA_IFP
    IfName(Option<SockaddrStorage>),
    /// RTA_IFA
    IfSockaddr(Option<SockaddrStorage>),
    /// RTA_AUTHOR
    RedirectAuthor(Option<SockaddrStorage>),
    /// RTA_BRD
    Broadcast(Option<SockaddrStorage>),
}

impl RouteSocketAddress {
    // Returns a new route socket address and number of bytes read from the buffer
    pub fn new(flag: AddressFlag, buf: &[u8]) -> Result<(Self, u8)> {
        // If buffer is empty, then the socket address is empty too, the backing buffer shouldn't
        // be advanced.
        if buf.is_empty() {
            return Ok((Self::with_sockaddr(flag, None)?, 0));
        }

        // to get the length and type of
        if buf.len() < std::mem::size_of::<sockaddr_hdr>() {
            return Err(Error::BufferTooSmall(
                "sockaddr buffer too small",
                buf.len(),
                std::mem::size_of::<sockaddr_hdr>(),
            ));
        }

        let addr_header_ptr = buf.as_ptr() as *const sockaddr_hdr;
        // SAFETY: Since `buf` is at least as long as a `sockaddr_hdr`, it's perfectly valid to
        // read from.
        let addr_header = unsafe { std::ptr::read(addr_header_ptr) };
        let saddr_len = addr_header.sa_len;
        if saddr_len == 0 {
            return Ok((Self::with_sockaddr(flag, None)?, 4));
        }

        if Into::<usize>::into(saddr_len) > buf.len() {
            return Err(Error::InvalidBuffer(buf.to_vec(), flag));
        }

        // SAFETY: the buffer is big enough for the sockaddr struct inside it, so accessing as a
        // `sockaddr` is valid.
        let saddr = unsafe {
            SockaddrStorage::from_raw(
                addr_header_ptr as *const nix::libc::sockaddr,
                Some(saddr_len.into()),
            )
        };

        Ok((Self::with_sockaddr(flag, saddr)?, saddr_len))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.inner() {
            None => vec![0u8; 4],
            Some(addr) => {
                let len = usize::try_from(addr.len()).unwrap();
                assert!(len >= 4);

                // The "serialized" socket addresses must be padded to be aligned to 4 bytes, with
                // the smallest size being 4 bytes.
                let buffer_size = len + len % 4;
                let mut buffer = vec![0u8; buffer_size];
                unsafe {
                    // SAFETY: copying conents of addr into buffer is safe, as long as addr.len()
                    // returns a correct size for the socket address pointer.
                    std::ptr::copy_nonoverlapping(
                        addr.as_ptr() as *const _,
                        buffer.as_mut_ptr(),
                        len,
                    );
                }
                buffer
            }
        }
    }

    pub fn address_flag(&self) -> AddressFlag {
        match &self {
            Self::Destination(_) => AddressFlag::RTA_DST,
            Self::Gateway(_) => AddressFlag::RTA_GATEWAY,
            Self::Netmask(_) => AddressFlag::RTA_NETMASK,
            Self::CloningMask(_) => AddressFlag::RTA_GENMASK,
            Self::IfName(_) => AddressFlag::RTA_IFP,
            Self::IfSockaddr(_) => AddressFlag::RTA_IFA,
            Self::RedirectAuthor(_) => AddressFlag::RTA_AUTHOR,
            Self::Broadcast(_) => AddressFlag::RTA_BRD,
        }
    }

    pub fn inner(&self) -> Option<&SockaddrStorage> {
        match &self {
            Self::Gateway(addr)
            | Self::Destination(addr)
            | Self::Netmask(addr)
            | Self::CloningMask(addr)
            | Self::IfName(addr)
            | Self::IfSockaddr(addr)
            | Self::RedirectAuthor(addr)
            | Self::Broadcast(addr) => addr.as_ref(),
        }
    }

    fn with_sockaddr(flag: AddressFlag, sockaddr: Option<SockaddrStorage>) -> Result<Self> {
        let constructor = match flag {
            AddressFlag::RTA_GATEWAY => Self::Gateway,
            AddressFlag::RTA_DST => Self::Destination,
            AddressFlag::RTA_NETMASK => Self::Netmask,
            AddressFlag::RTA_GENMASK => Self::CloningMask,
            AddressFlag::RTA_IFP => Self::IfName,
            AddressFlag::RTA_IFA => Self::IfSockaddr,
            AddressFlag::RTA_AUTHOR => Self::RedirectAuthor,
            AddressFlag::RTA_BRD => Self::Broadcast,
            unknown => return Err(Error::UnknownAddressFlag(unknown.bits())),
        };

        Ok(constructor(sockaddr))
    }

    pub fn interface_index(&self) -> Option<u16> {
        match self {
            Self::IfName(Some(iface)) => {
                let index = iface.as_link_addr()?.ifindex();
                Some(
                    u16::try_from(index)
                        .expect("interface indexes actually are u16s, nix is just *interesting*"),
                )
            }
            _ => None,
        }
    }
}

/// Route socket addreses should be ordered by their corresponding address flag when a route
/// message is constructed
impl std::cmp::PartialOrd for RouteSocketAddress {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address_flag().partial_cmp(&other.address_flag())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct sockaddr_hdr {
    sa_len: u8,
    sa_family: libc::sa_family_t,
    padding: u16,
}
