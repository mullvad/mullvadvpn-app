use super::{
    saddr_to_ipv4, saddr_to_ipv6, AddressFlag, Error, Result, RouteSockAddrIterator,
    RouteSocketAddress,
};
use std::{
    collections::BTreeMap,
    net::IpAddr,
    os::raw::{c_int, c_uchar, c_ushort},
};

#[derive(Debug)]
#[repr(C)]
struct ifa_msghdr {
    ifam_msglen: c_ushort,
    ifam_version: c_uchar,
    ifam_type: c_uchar,
    ifam_addrs: c_int,
    ifam_flags: c_int,
    ifam_index: c_ushort,
    ifam_metric: c_int,
}

#[derive(Debug)]
pub struct AddressMessage {
    sockaddrs: BTreeMap<AddressFlag, RouteSocketAddress>,
    interface_index: u16,
}

impl AddressMessage {
    pub fn index(&self) -> u16 {
        self.interface_index
    }

    pub fn address(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_IFP)
            .or_else(|| self.get_address(&AddressFlag::RTA_IFA))
            .ok_or(Error::NoInterfaceAddress)
    }

    fn get_address(&self, address_flag: &AddressFlag) -> Option<IpAddr> {
        let addr = self.sockaddrs.get(address_flag)?;
        saddr_to_ipv4(addr.inner()?)
            .map(IpAddr::from)
            .or_else(|| saddr_to_ipv6(addr.inner()?).map(IpAddr::from))
    }

    pub fn netmask(&self) -> Result<IpAddr> {
        self.get_address(&AddressFlag::RTA_NETMASK)
            .ok_or(Error::NoNetmaskAddress)
    }

    pub fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const HEADER_SIZE: usize = std::mem::size_of::<ifa_msghdr>();
        if HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall(
                "ifa_msghdr",
                buffer.len(),
                HEADER_SIZE,
            ));
        }

        // SAFETY: buffer is pointing to enough memory to contain a valid value for ifa_msghdr
        let header: ifa_msghdr = unsafe { std::ptr::read_unaligned(buffer.as_ptr() as *const _) };

        let msg_len = usize::from(header.ifam_msglen);
        if msg_len > buffer.len() {
            return Err(Error::BufferTooSmall(
                "Message is shorter than it's msg_len indicates",
                msg_len,
                buffer.len(),
            ));
        }

        let payload = &buffer[HEADER_SIZE..std::cmp::min(msg_len, buffer.len())];

        let address_flags = AddressFlag::from_bits(header.ifam_addrs)
            .ok_or(Error::UnknownAddressFlag(header.ifam_addrs))?;

        let sockaddrs = RouteSockAddrIterator::new(payload, address_flags)
            .map(|addr| addr.map(|addr| (addr.address_flag(), addr)))
            .collect::<Result<BTreeMap<_, _>>>()?;

        Ok(Self {
            sockaddrs,
            interface_index: header.ifam_index,
        })
    }
}
