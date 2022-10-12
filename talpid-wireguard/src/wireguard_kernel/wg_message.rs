use super::{super::config::Config, parsers, Error};
use byteorder::{ByteOrder, NativeEndian};
use ipnetwork::IpNetwork;
use netlink_packet_core::{
    NetlinkDeserializable, NetlinkHeader, NetlinkPayload, NetlinkSerializable,
};
use netlink_packet_utils::{
    nla::{Nla, NlaBuffer, NlasIterator, NLA_F_NESTED},
    traits::{Emitable, Parseable},
    DecodeError,
};
use nix::sys::{socket::InetAddr, time::TimeSpec};
use std::{ffi::CString, io::Write, mem, net::IpAddr};

/// WireGuard netlink constants
mod constants {
    #![allow(dead_code)]
    pub const WG_GENL_VERSION: u8 = 1;

    /// Command constants
    pub const WG_CMD_GET_DEVICE: u8 = 0;
    pub const WG_CMD_SET_DEVICE: u8 = 1;

    // wgdevice_flag
    pub const WGDEVICE_F_REPLACE_PEERS: u32 = 1 << 0;

    // wgdevice_attribute
    pub const WGDEVICE_A_UNSPEC: u16 = 0;
    pub const WGDEVICE_A_IFINDEX: u16 = 1;
    pub const WGDEVICE_A_IFNAME: u16 = 2;
    pub const WGDEVICE_A_PRIVATE_KEY: u16 = 3;
    pub const WGDEVICE_A_PUBLIC_KEY: u16 = 4;
    pub const WGDEVICE_A_FLAGS: u16 = 5;
    pub const WGDEVICE_A_LISTEN_PORT: u16 = 6;
    pub const WGDEVICE_A_FWMARK: u16 = 7;
    pub const WGDEVICE_A_PEERS: u16 = 8;

    // wgpeer_flag
    pub const WGPEER_F_REMOVE_ME: u32 = 1 << 0;
    pub const WGPEER_F_REPLACE_ALLOWEDIPS: u32 = 1 << 1;
    pub const WGPEER_F_UPDATE_ONLY: u32 = 1 << 2;

    // wgpeer_attribute
    pub const WGPEER_A_UNSPEC: u16 = 0;
    pub const WGPEER_A_PUBLIC_KEY: u16 = 1;
    pub const WGPEER_A_PRESHARED_KEY: u16 = 2;
    pub const WGPEER_A_FLAGS: u16 = 3;
    pub const WGPEER_A_ENDPOINT: u16 = 4;
    pub const WGPEER_A_PERSISTENT_KEEPALIVE_INTERVAL: u16 = 5;
    pub const WGPEER_A_LAST_HANDSHAKE_TIME: u16 = 6;
    pub const WGPEER_A_RX_BYTES: u16 = 7;
    pub const WGPEER_A_TX_BYTES: u16 = 8;
    pub const WGPEER_A_ALLOWEDIPS: u16 = 9;
    pub const WGPEER_A_PROTOCOL_VERSION: u16 = 10;

    // wgallowedip_attribute
    pub const WGALLOWEDIP_A_UNSPEC: u16 = 0;
    pub const WGALLOWEDIP_A_FAMILY: u16 = 1;
    pub const WGALLOWEDIP_A_IPADDR: u16 = 2;
    pub const WGALLOWEDIP_A_CIDR_MASK: u16 = 3;
}

use constants::*;
pub use constants::{WG_CMD_GET_DEVICE, WG_CMD_SET_DEVICE};

type PrivateKey = [u8; 32];
type PublicKey = [u8; 32];
type PresharedKey = [u8; 32];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeviceMessage {
    pub nlas: Vec<DeviceNla>,
    pub message_type: u16,
    pub command: u8,
}

impl DeviceMessage {
    pub fn reset_config(message_type: u16, interface_index: u32, config: &Config) -> DeviceMessage {
        let mut peers = vec![];

        for peer in config.peers.iter() {
            let peer_endpoint = InetAddr::from_std(&peer.endpoint);
            let allowed_ips = peer.allowed_ips.iter().map(From::from).collect();
            let mut peer_nlas = vec![
                PeerNla::PublicKey(*peer.public_key.as_bytes()),
                PeerNla::Endpoint(peer_endpoint),
                PeerNla::AllowedIps(allowed_ips),
                PeerNla::Flags(WGPEER_F_REPLACE_ALLOWEDIPS),
            ];
            if let Some(psk) = peer.psk.as_ref() {
                peer_nlas.push(PeerNla::PresharedKey(*psk.as_bytes()));
            }
            peers.push(PeerMessage(peer_nlas));
        }

        let nlas = vec![
            DeviceNla::IfIndex(interface_index),
            DeviceNla::ListenPort(0),
            DeviceNla::Fwmark(config.fwmark.unwrap_or(0)),
            DeviceNla::PrivateKey(config.tunnel.private_key.to_bytes()),
            DeviceNla::Flags(WGDEVICE_F_REPLACE_PEERS),
            DeviceNla::Peers(peers),
        ];

        Self {
            nlas,
            message_type,
            command: WG_CMD_SET_DEVICE,
        }
    }

    pub fn get_by_name(message_type: u16, name: String) -> Result<Self, Error> {
        let c_name = CString::new(name).map_err(|_| Error::InterfaceName)?;
        if c_name.as_bytes_with_nul().len() > libc::IFNAMSIZ {
            return Err(Error::InterfaceName);
        }

        Ok(Self {
            message_type,
            nlas: vec![DeviceNla::IfName(c_name)],
            command: WG_CMD_GET_DEVICE,
        })
    }

    pub fn get_by_index(message_type: u16, index: u32) -> Self {
        Self {
            message_type,
            nlas: vec![DeviceNla::IfIndex(index)],
            command: WG_CMD_GET_DEVICE,
        }
    }

    // All WireGuard netlink messages should start with a libc::genlmsghdr, for which the first
    // byte contains the command.
    fn read_genlmsghdr(buff: &[u8]) -> Result<u8, Error> {
        if buff.len() < mem::size_of::<libc::genlmsghdr>() {
            return Err(Error::Truncated);
        }

        let cmd = buff[0];
        if cmd == WG_CMD_GET_DEVICE || cmd == WG_CMD_SET_DEVICE {
            Ok(cmd)
        } else {
            Err(Error::UnnkownWireguardCommmand(cmd))
        }
    }
}

impl NetlinkSerializable for DeviceMessage {
    fn message_type(&self) -> u16 {
        self.message_type
    }

    fn buffer_len(&self) -> usize {
        // add the genlmsghdr
        mem::size_of::<libc::genlmsghdr>() +
            // size of all of the NLAs
            self.nlas.as_slice().buffer_len()
    }

    fn serialize(&self, mut buffer: &mut [u8]) {
        let command_buf = [self.command, WG_GENL_VERSION, 0u8, 0u8];
        let _ = buffer.write(&command_buf).unwrap();
        self.nlas.as_slice().emit(buffer)
    }
}

impl From<DeviceMessage> for NetlinkPayload<DeviceMessage> {
    fn from(msg: DeviceMessage) -> Self {
        NetlinkPayload::InnerMessage(msg)
    }
}

impl NetlinkDeserializable for DeviceMessage {
    type Error = Error;
    fn deserialize(header: &NetlinkHeader, payload: &[u8]) -> Result<DeviceMessage, Self::Error> {
        let command = Self::read_genlmsghdr(payload)?;
        let new_payload = &payload[mem::size_of::<libc::genlmsghdr>()..];
        let mut nlas = vec![];
        for buf in NlasIterator::new(new_payload) {
            nlas.push(DeviceNla::parse(&buf.map_err(Error::Decode)?).map_err(Error::Decode)?);
        }

        Ok(DeviceMessage {
            nlas,
            command,
            message_type: header.message_type,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DeviceNla {
    IfIndex(u32),
    IfName(CString),
    Flags(u32),
    PrivateKey(PrivateKey),
    PublicKey(PublicKey),
    ListenPort(u16),
    Fwmark(u32),
    Peers(Vec<PeerMessage>),
    Unspec(Vec<u8>),
}

impl Nla for DeviceNla {
    fn value_len(&self) -> usize {
        use DeviceNla::*;
        match self {
            IfIndex(_) | Fwmark(_) | Flags(_) => 4,
            IfName(name) => name.as_bytes_with_nul().len(),
            PrivateKey(key) | PublicKey(key) => key.len(),
            ListenPort(_) => 2,
            Peers(peers) => peers.as_slice().buffer_len(),
            Unspec(payload) => payload.len(),
        }
    }

    fn kind(&self) -> u16 {
        use DeviceNla::*;
        match self {
            IfIndex(_) => WGDEVICE_A_IFINDEX,
            IfName(_) => WGDEVICE_A_IFNAME,
            PrivateKey(_) => WGDEVICE_A_PRIVATE_KEY,
            PublicKey(_) => WGDEVICE_A_PUBLIC_KEY,
            Flags(_) => WGDEVICE_A_FLAGS,
            ListenPort(_) => WGDEVICE_A_LISTEN_PORT,
            Fwmark(_) => WGDEVICE_A_FWMARK,
            Peers(_) => WGDEVICE_A_PEERS | NLA_F_NESTED,
            Unspec(_) => WGDEVICE_A_UNSPEC,
        }
    }

    fn emit_value(&self, mut buffer: &mut [u8]) {
        use DeviceNla::*;
        match self {
            IfIndex(value) | Fwmark(value) | Flags(value) => {
                NativeEndian::write_u32(buffer, *value)
            }
            IfName(interface_name) => {
                let _ = buffer
                    .write(interface_name.as_bytes_with_nul())
                    .expect("Failed to write interface name");
            }
            PrivateKey(key) | PublicKey(key) => {
                let _ = buffer.write(key).expect("Failed to write key");
            }
            ListenPort(port) => NativeEndian::write_u16(buffer, *port),
            Peers(peers) => {
                peers.as_slice().emit(buffer);
            }
            Unspec(payload) => {
                let _ = buffer.write(payload).expect("Failed to write ");
            }
        }
    }
}

impl<'a, T: AsRef<[u8]> + 'a + ?Sized + core::fmt::Debug> Parseable<NlaBuffer<&'a T>>
    for DeviceNla
{
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        use DeviceNla::*;
        let value = buf.value();
        let kind = buf.kind();
        let nla = match kind {
            WGDEVICE_A_IFINDEX => IfIndex(parsers::parse_u32(value)?),
            WGDEVICE_A_IFNAME => IfName(parsers::parse_cstring(value)?),
            WGDEVICE_A_PRIVATE_KEY => PrivateKey(parsers::parse_wg_key(value)?),
            WGDEVICE_A_PUBLIC_KEY => PublicKey(parsers::parse_wg_key(value)?),
            WGDEVICE_A_FLAGS => Flags(parsers::parse_u32(value)?),
            WGDEVICE_A_LISTEN_PORT => ListenPort(parsers::parse_u16(value)?),
            WGDEVICE_A_FWMARK => Fwmark(parsers::parse_u32(value)?),
            WGDEVICE_A_PEERS => {
                let peers = NlasIterator::new(value)
                    .map(|nla_bytes| {
                        let buf = nla_bytes?;
                        let val = buf.value();
                        PeerMessage::parse(val)
                    })
                    .collect::<Result<Vec<PeerMessage>, DecodeError>>()?;
                Peers(peers)
            }
            WGDEVICE_A_UNSPEC => Unspec(value.to_vec()),
            _ => {
                return Err(format!("Unexpected device attribute kind: {}", buf.kind()).into());
            }
        };
        Ok(nla)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PeerMessage(pub Vec<PeerNla>);

impl PeerMessage {
    fn parse(payload: &[u8]) -> Result<Self, DecodeError> {
        let mut nlas = vec![];

        let nla_iter = NlasIterator::new(&payload);
        for buffer in nla_iter {
            nlas.push(PeerNla::parse(&buffer?)?)
        }
        Ok(Self(nlas))
    }
}

impl<'a, T: AsRef<[u8]> + 'a + ?Sized> Parseable<NlaBuffer<&'a T>> for PeerMessage {
    fn parse(payload: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        Ok(Self(
            NlasIterator::new(&payload.into_inner())
                .map(|buffer| PeerNla::parse(&buffer?))
                .collect::<Result<Vec<PeerNla>, DecodeError>>()?,
        ))
    }
}

impl Nla for PeerMessage {
    fn value_len(&self) -> usize {
        self.0.as_slice().buffer_len()
    }

    fn kind(&self) -> u16 {
        NLA_F_NESTED
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        self.0.as_slice().emit(buffer);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PeerNla {
    Unspec(Vec<u8>),
    PublicKey(PublicKey),
    PresharedKey(PresharedKey),
    Flags(u32),
    Endpoint(InetAddr),
    PersistentKeepaliveInterval(u16),
    LastHandshakeTime(TimeSpec),
    RxBytes(u64),
    TxBytes(u64),
    AllowedIps(Vec<AllowedIpMessage>),
    ProtocolVersion(u32),
}

impl Nla for PeerNla {
    fn value_len(&self) -> usize {
        use PeerNla::*;
        match self {
            PublicKey(key) | PresharedKey(key) => key.len(),
            Endpoint(endpoint) => match &endpoint {
                InetAddr::V4(_) => mem::size_of::<libc::sockaddr_in>(),
                InetAddr::V6(_) => mem::size_of::<libc::sockaddr_in6>(),
            },
            PersistentKeepaliveInterval(_) => 2,
            LastHandshakeTime(_) => mem::size_of::<libc::timespec>(),
            RxBytes(_) | TxBytes(_) => 8,
            AllowedIps(ips) => ips.as_slice().buffer_len(),
            Flags(_) | ProtocolVersion(_) => 4,
            Unspec(payload) => payload.len(),
        }
    }

    fn kind(&self) -> u16 {
        use PeerNla::*;
        match self {
            PublicKey(_) => WGPEER_A_PUBLIC_KEY,
            PresharedKey(_) => WGPEER_A_PRESHARED_KEY,
            Flags(_) => WGPEER_A_FLAGS,
            Endpoint(_) => WGPEER_A_ENDPOINT,
            PersistentKeepaliveInterval(_) => WGPEER_A_PERSISTENT_KEEPALIVE_INTERVAL,
            LastHandshakeTime(_) => WGPEER_A_LAST_HANDSHAKE_TIME,
            RxBytes(_) => WGPEER_A_RX_BYTES,
            TxBytes(_) => WGPEER_A_TX_BYTES,
            AllowedIps(_) => WGPEER_A_ALLOWEDIPS | NLA_F_NESTED,
            ProtocolVersion(_) => WGPEER_A_PROTOCOL_VERSION,
            Unspec(_) => WGPEER_A_UNSPEC,
        }
    }

    fn emit_value(&self, mut buffer: &mut [u8]) {
        use PeerNla::*;
        match self {
            PublicKey(key) | PresharedKey(key) => {
                let _ = buffer.write(key).expect("Buffer too small for a key");
            }
            Flags(value) | ProtocolVersion(value) => NativeEndian::write_u32(buffer, *value),
            Endpoint(endpoint) => match &endpoint {
                InetAddr::V4(sockaddr_in) => {
                    // SAFETY: `sockaddr_in` has no padding bytes
                    buffer
                        .write_all(unsafe { struct_as_slice(sockaddr_in) })
                        .expect("Buffer too small for sockaddr_in");
                }
                InetAddr::V6(sockaddr_in6) => {
                    // SAFETY: `sockaddr_in` has no padding bytes
                    buffer
                        .write_all(unsafe { struct_as_slice(sockaddr_in6) })
                        .expect("Buffer too small for sockaddr_in6");
                }
            },
            PersistentKeepaliveInterval(interval) => {
                NativeEndian::write_u16(buffer, *interval);
            }
            LastHandshakeTime(last_handshake) => {
                let timespec: &libc::timespec = last_handshake.as_ref();
                // SAFETY: `timespec` has no padding bytes
                buffer
                    .write_all(unsafe { struct_as_slice(timespec) })
                    .expect("Buffer too small for timespec");
            }
            RxBytes(num_bytes) | TxBytes(num_bytes) => NativeEndian::write_u64(buffer, *num_bytes),
            AllowedIps(ips) => ips.as_slice().emit(buffer),
            Unspec(payload) => {
                let _ = buffer
                    .write(payload)
                    .expect("Buffer too small for unspecified payload");
            }
        }
    }
}

impl<'a, T: AsRef<[u8]> + 'a + ?Sized> Parseable<NlaBuffer<&'a T>> for PeerNla {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        use PeerNla::*;
        let value = buf.value();
        let nla = match buf.kind() {
            WGPEER_A_PUBLIC_KEY => PublicKey(parsers::parse_wg_key(value)?),
            WGPEER_A_PRESHARED_KEY => PresharedKey(parsers::parse_wg_key(value)?),
            WGPEER_A_FLAGS => Flags(parsers::parse_u32(value)?),
            WGPEER_A_ENDPOINT => Endpoint(parsers::parse_inet_sockaddr(value)?),
            WGPEER_A_PERSISTENT_KEEPALIVE_INTERVAL => {
                PersistentKeepaliveInterval(parsers::parse_u16(value)?)
            }

            WGPEER_A_LAST_HANDSHAKE_TIME => LastHandshakeTime(parsers::parse_timespec(value)?),
            WGPEER_A_RX_BYTES => RxBytes(parsers::parse_u64(value)?),
            WGPEER_A_TX_BYTES => TxBytes(parsers::parse_u64(value)?),
            WGPEER_A_ALLOWEDIPS => {
                let nlas = NlasIterator::new(value)
                    .map(|nla_buffer| AllowedIpMessage::parse(&nla_buffer?))
                    .collect::<Result<Vec<_>, DecodeError>>()?;

                AllowedIps(nlas)
            }
            WGPEER_A_PROTOCOL_VERSION => ProtocolVersion(parsers::parse_u32(value)?),
            WGPEER_A_UNSPEC => Unspec(value.to_vec()),
            _ => {
                return Err(format!("Unexpected peer attribute kind: {}", buf.kind()).into());
            }
        };
        Ok(nla)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AllowedIpMessage(Vec<AllowedIpNla>);

impl From<&IpNetwork> for AllowedIpMessage {
    fn from(ip: &IpNetwork) -> Self {
        use AllowedIpNla::*;
        let address_family = if ip.is_ipv4() {
            libc::AF_INET
        } else {
            libc::AF_INET6
        };

        AllowedIpMessage(vec![
            AddressFamily(address_family as u16),
            CidrMask(ip.prefix()),
            IpAddr(ip.ip()),
        ])
    }
}

impl Nla for AllowedIpMessage {
    fn value_len(&self) -> usize {
        self.0.as_slice().buffer_len()
    }

    fn kind(&self) -> u16 {
        NLA_F_NESTED
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        self.0.as_slice().emit(buffer);
    }
}

impl<'a, T: AsRef<[u8]> + 'a + ?Sized> Parseable<NlaBuffer<&'a T>> for AllowedIpMessage {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        let nlas = NlasIterator::new(buf.value())
            .map(|buffer| AllowedIpNla::parse(&buffer?))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AllowedIpMessage(nlas))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AllowedIpNla {
    AddressFamily(u16),
    IpAddr(IpAddr),
    CidrMask(u8),
    Unspec(Vec<u8>),
}

impl Nla for AllowedIpNla {
    fn value_len(&self) -> usize {
        use AllowedIpNla::*;
        match &self {
            AddressFamily(_) => 2,
            IpAddr(addr) => ip_addr_to_bytes(addr).len(),
            CidrMask(_) => 1,
            Unspec(payload) => payload.len(),
        }
    }

    fn kind(&self) -> u16 {
        use AllowedIpNla::*;
        match &self {
            AddressFamily(_) => WGALLOWEDIP_A_FAMILY,
            IpAddr(_) => WGALLOWEDIP_A_IPADDR,
            CidrMask(_) => WGALLOWEDIP_A_CIDR_MASK,
            Unspec(_) => WGALLOWEDIP_A_UNSPEC,
        }
    }

    fn emit_value(&self, mut buffer: &mut [u8]) {
        use AllowedIpNla::*;
        match self {
            AddressFamily(af) => {
                NativeEndian::write_u16(buffer, *af);
            }
            IpAddr(ip_addr) => {
                buffer
                    .write_all(&ip_addr_to_bytes(ip_addr))
                    .expect("Buffer too small for AllowedIpNla::IpAddr");
            }
            CidrMask(cidr_mask) => buffer[0] = *cidr_mask,
            Unspec(payload) => {
                let _ = buffer
                    .write(payload)
                    .expect("Buffer too small for unspec payload");
            }
        }
    }
}

impl<'a, T: AsRef<[u8]> + 'a + ?Sized> Parseable<NlaBuffer<&'a T>> for AllowedIpNla {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        use AllowedIpNla::*;
        let value = buf.value();
        let nla = match buf.kind() {
            WGALLOWEDIP_A_FAMILY => AddressFamily(parsers::parse_u16(value)?),
            WGALLOWEDIP_A_IPADDR => IpAddr(parsers::parse_ip_addr(value)?),
            WGALLOWEDIP_A_CIDR_MASK => CidrMask(parsers::parse_u8(value)?),
            WGALLOWEDIP_A_UNSPEC => Unspec(value.to_vec()),
            _ => Err(format!(
                "Unexpected allowed IP attribute kind: {}",
                buf.kind()
            ))?,
        };
        Ok(nla)
    }
}

/// Returns a byte slice over the memory used by `t`.
///
/// # Safety
///
/// The returned slice includes any padding bytes of `t`. Padding bytes are uninitialized
/// data and it is undefined behavior for a `u8` to be uninitialized. Only call this method
/// on `T`s without padding.
unsafe fn struct_as_slice<T: Sized>(t: &T) -> &[u8] {
    let size = mem::size_of::<T>();
    let ptr = t as *const T as *const u8;
    // SAFETY: The memory from `ptr` and `size` bytes forward is always the same as the struct.
    // The caller is responsible for not using this with structs containing padding.
    std::slice::from_raw_parts(ptr, size)
}

fn ip_addr_to_bytes(addr: &IpAddr) -> Vec<u8> {
    match addr {
        IpAddr::V4(addr) => addr.octets().to_vec(),
        IpAddr::V6(addr) => addr.octets().to_vec(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nix::sys::time::TimeValLike;
    use std::net::Ipv4Addr;

    #[test]
    fn deserialize_netlink_message() {
        #[rustfmt::skip]
        let payload = vec![
            0x00, 0x01, 0x00, 0x00,
            // 6 bytes of WGDEVICE_A_LISTEN_PORT 51820 + 2 bytes of padding
            0x06, 0x00, 0x06, 0x00, 0x6c, 0xca, 0x00, 0x00,
            // 8 bytes of WGDEVICE_A_FWMARK 0
            0x08, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00,
            // 8 bytes of WGDEVIEC_A_IFINDEX 320
            0x08, 0x00, 0x01, 0x00, 0x40, 0x01, 0x00, 0x00,
            // 12 bytes of WGDEVICE_A_IFNAME "wg-test\0"
            0x0c, 0x00, 0x02, 0x00, 0x77, 0x67, 0x2d, 0x74, 0x65, 0x73, 0x74, 0x00,
            // 36 bytes of WGDEVICE_A_PRIVATE_KEY OEf0rWXfVRarrw8nNbTBxkk3NTu8GjRKrbMW1aFH/H0=
            0x24, 0x00, 0x03, 0x00, 0x38, 0x47, 0xf4, 0xad, 0x65, 0xdf, 0x55, 0x16, 0xab, 0xaf,
            0x0f, 0x27, 0x35, 0xb4, 0xc1, 0xc6, 0x49, 0x37, 0x35, 0x3b, 0xbc, 0x1a, 0x34, 0x4a,
            0xad, 0xb3, 0x16, 0xd5, 0xa1, 0x47, 0xfc, 0x7d,
            // 36 bytes of WGDEVICE_A_PUBLIC_KEY Ztqy3r8VO1N8tHwpWwqGx1S6G9o12BRdy1JESr2OYzs=
            0x24, 0x00, 0x04, 0x00, 0x66, 0xda, 0xb2, 0xde, 0xbf, 0x15, 0x3b, 0x53, 0x7c, 0xb4,
            0x7c, 0x29, 0x5b, 0x0a, 0x86, 0xc7, 0x54, 0xba, 0x1b, 0xda, 0x35, 0xd8, 0x14, 0x5d,
            0xcb, 0x52, 0x44, 0x4a, 0xbd, 0x8e, 0x63, 0x3b,
            // 380 bytes of WGDEVICE_A_PEERS
            0x7c, 0x01, 0x08, 0x80,
                  // 188 bytes of WGPEER attributes
                  0xbc, 0x00, 0x00, 0x80,
                        // 36 bytes of WGPEER_A_PUBLIC_KEY IOBEBReIZ+XOOyLn14vW7FBRuweaxfskq5wwSZEvhjY=
                        0x24, 0x00, 0x01, 0x00, 0x20, 0xe0, 0x44, 0x05, 0x17, 0x88, 0x67, 0xe5,
                        0xce, 0x3b, 0x22, 0xe7, 0xd7, 0x8b, 0xd6, 0xec, 0x50, 0x51, 0xbb, 0x07,
                        0x9a, 0xc5, 0xfb, 0x24, 0xab, 0x9c, 0x30, 0x49, 0x91, 0x2f, 0x86, 0x36,
                        // 36 bytes of WGPEER_A_PRESHARED_KEY (all zeroes)
                        0x24, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 20 bytes of WGPEER_A_LAST_HANDSHAKE_TIME 0
                        0x14, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 6 bytes of WGPEER_A_PERSISTENT_KEEPALIVE_INTERVAL 0
                        0x06, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 12 bytes of WGPEER_A_TX_BYTES 0
                        0x0c, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 12 bytes of WGPEER_A_RX_BYTES 0
                        0x0c, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 8 bytes of WGPEER_A_PROTOCOL_VERSION 1
                        0x08, 0x00, 0x0a, 0x00, 0x01, 0x00, 0x00, 0x00,
                        // 20 bytes of WGPEER_A_ENDPOINT 192.168.39.2:9797
                        0x14, 0x00, 0x04, 0x00, 0x02, 0x00, 0x26, 0x45, 0xc0, 0xa8, 0x28, 0x01,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 32 bytes of WGPEER_A_ALLOWEDIPS
                        0x20, 0x00, 0x09, 0x80,
                              // 28 bytes of WGALLOWDIP_A_*
                              0x1c, 0x00,0x00, 0x80,
                                    // 5 bytes of WGALLOWEDIP_A_CIDR_MASK + 3 bytes of padding 32
                                    0x05, 0x00, 0x03, 0x00, 0x20, 0x00, 0x00, 0x00,
                                    // 6 bytes of WGALLOWEDIP_A_FAMILY + 2 bytes of padding 2 (IPv4)
                                    0x06, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00,
                                    // 8 bytes of WGALLOWEDIP_A_IPADDR 192.168.40.1
                                    0x08, 0x00, 0x02, 0x00, 0xc0, 0xa8, 0x27, 0x01,
                  // 188 bvytes of WGPEER attributes
                  0xbc, 0x00, 0x00, 0x80,
                        // 36 bytes of WGPEER_A_PUBLIC_KEY
                        0x24, 0x00, 0x01, 0x00, 0xf4, 0x1c, 0xce, 0x0c, 0x4f, 0x24, 0x58, 0xb7,
                        0xc2, 0x9d, 0x36, 0x26, 0x36, 0xb7, 0x7f, 0x20, 0x8e, 0x18, 0xfb, 0x9e,
                        0xd9, 0x38, 0x0c, 0x92, 0xd0, 0x15, 0x84, 0x9d, 0xa2, 0x44, 0x02, 0x2c,
                        // 36 bytes of WGPEER_A_PRESHARED_KEY
                        0x24, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 20 bytes of WGPEER_A_LAST_HANDSHAKE_TIME
                        0x14, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 6 bytes of WGPEER_A_PERSISTENT_KEEPALIVE_INTERVAL + 2 bytes of padding
                        0x06, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 12 bytes of WGPEER_A_TX_BYTES
                        0x0c, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 12 bytes of WGPEER_A_RX_BYTES
                        0x0c, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 8 bytes of WGPEER_A_PROTOCOL_VERSION
                        0x08, 0x00, 0x0a, 0x00, 0x01, 0x00, 0x00, 0x00,
                        // 20 bytes of WGPEER_A_ENDPOINT
                        0x14, 0x00, 0x04, 0x00, 0x02, 0x00, 0x26, 0x45, 0xc0, 0xa8, 0x28, 0x02,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        // 32 bytes of WGPEER_A_ALLOWEDIPS
                        0x20, 0x00, 0x09, 0x80,
                              // 28 bytes of WGALLOWDIP_A_*
                              0x1c, 0x00, 0x00, 0x80,
                                    // 5 bytes of WGALLOWEDIP_A_CIDR_MASK + 3 bytes of padding 32
                                    0x05, 0x00, 0x03, 0x00, 0x20, 0x00, 0x00, 0x00,
                                    // 6 bytes of WGALLOWEDIP_A_FAMILY + 2 bytes of padding 2 (IPv4)
                                    0x06, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00,
                                    // 8 bytes of WGALLOWEDIP_A_IPADDR 192.168.40.2
                                    0x08, 0x00, 0x02, 0x00, 0xc0, 0xa8, 0x27, 0x02,
        ];
        let header = NetlinkHeader {
            length: payload.len() as u32,
            message_type: 0,
            flags: 0,
            sequence_number: 0,
            port_number: 0,
        };
        let message = DeviceMessage::deserialize(&header, &payload).unwrap();

        let mut serialized_message = vec![0u8; payload.len()];

        message.serialize(&mut serialized_message);

        assert_eq!(message, sample_get_message());
        assert_eq!(&payload, &serialized_message)
    }

    fn sample_get_message() -> DeviceMessage {
        use AllowedIpNla::*;
        use DeviceNla::*;
        use PeerNla::*;

        let if_name = CString::new(b"wg-test".to_vec()).unwrap();

        let peer_1 = PeerMessage(
            [
                PeerNla::PublicKey([
                    32, 224, 68, 5, 23, 136, 103, 229, 206, 59, 34, 231, 215, 139, 214, 236, 80,
                    81, 187, 7, 154, 197, 251, 36, 171, 156, 48, 73, 145, 47, 134, 54,
                ]),
                PeerNla::PresharedKey([
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ]),
                LastHandshakeTime(TimeSpec::seconds(0)),
                PersistentKeepaliveInterval(0),
                TxBytes(0),
                RxBytes(0),
                ProtocolVersion(1),
                Endpoint(InetAddr::from_std(&"192.168.40.1:9797".parse().unwrap())),
                AllowedIps(
                    [AllowedIpMessage(
                        [
                            CidrMask(32),
                            AddressFamily(2),
                            IpAddr(Ipv4Addr::new(192, 168, 39, 1).into()),
                        ]
                        .to_vec(),
                    )]
                    .to_vec()
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );

        let peer_2 = PeerMessage(
            [
                PeerNla::PublicKey([
                    244, 28, 206, 12, 79, 36, 88, 183, 194, 157, 54, 38, 54, 183, 127, 32, 142, 24,
                    251, 158, 217, 56, 12, 146, 208, 21, 132, 157, 162, 68, 2, 44,
                ]),
                PresharedKey([
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ]),
                LastHandshakeTime(TimeSpec::seconds(0)),
                PersistentKeepaliveInterval(0),
                TxBytes(0),
                RxBytes(0),
                ProtocolVersion(1),
                Endpoint(InetAddr::from_std(&"192.168.40.2:9797".parse().unwrap())),
                AllowedIps(
                    [AllowedIpMessage(
                        vec![
                            CidrMask(32),
                            AddressFamily(2),
                            IpAddr(Ipv4Addr::new(192, 168, 39, 2).into()),
                        ]
                        .to_vec(),
                    )]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );

        DeviceMessage {
            command: WG_CMD_GET_DEVICE,
            message_type: 0,
            nlas: [
                ListenPort(51820),
                Fwmark(0),
                IfIndex(320),
                IfName(if_name),
                PrivateKey([
                    56, 71, 244, 173, 101, 223, 85, 22, 171, 175, 15, 39, 53, 180, 193, 198, 73,
                    55, 53, 59, 188, 26, 52, 74, 173, 179, 22, 213, 161, 71, 252, 125,
                ]),
                DeviceNla::PublicKey([
                    102, 218, 178, 222, 191, 21, 59, 83, 124, 180, 124, 41, 91, 10, 134, 199, 84,
                    186, 27, 218, 53, 216, 20, 93, 203, 82, 68, 74, 189, 142, 99, 59,
                ]),
                Peers([peer_1, peer_2].to_vec()),
            ]
            .to_vec(),
        }
    }

    pub fn sample_set_message() -> DeviceMessage {
        use AllowedIpNla::*;
        use DeviceNla::*;
        use PeerNla::*;

        let if_name = CString::new("wg-test".to_string()).unwrap();

        let peer_1 = PeerMessage(
            [
                PeerNla::PublicKey([
                    32, 224, 68, 5, 23, 136, 103, 229, 206, 59, 34, 231, 215, 139, 214, 236, 80,
                    81, 187, 7, 154, 197, 251, 36, 171, 156, 48, 73, 145, 47, 134, 54,
                ]),
                Endpoint(InetAddr::from_std(&"192.168.40.1:9797".parse().unwrap())),
                PeerNla::Flags(WGPEER_F_REPLACE_ALLOWEDIPS),
                AllowedIps(
                    [AllowedIpMessage(
                        [
                            AddressFamily(2),
                            IpAddr(Ipv4Addr::new(192, 168, 39, 1).into()),
                            CidrMask(32),
                        ]
                        .to_vec(),
                    )]
                    .to_vec()
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );

        let peer_2 = PeerMessage(
            [
                PeerNla::PublicKey([
                    244, 28, 206, 12, 79, 36, 88, 183, 194, 157, 54, 38, 54, 183, 127, 32, 142, 24,
                    251, 158, 217, 56, 12, 146, 208, 21, 132, 157, 162, 68, 2, 44,
                ]),
                Endpoint(InetAddr::from_std(&"192.168.40.2:9797".parse().unwrap())),
                PeerNla::Flags(WGPEER_F_REPLACE_ALLOWEDIPS),
                AllowedIps(
                    [AllowedIpMessage(
                        vec![
                            AddressFamily(2),
                            IpAddr(Ipv4Addr::new(192, 168, 39, 2).into()),
                            CidrMask(32),
                        ]
                        .to_vec(),
                    )]
                    .to_vec(),
                ),
            ]
            .to_vec(),
        );

        DeviceMessage {
            command: WG_CMD_SET_DEVICE,
            message_type: 0,
            nlas: [
                IfName(if_name),
                PrivateKey([
                    56, 71, 244, 173, 101, 223, 85, 22, 171, 175, 15, 39, 53, 180, 193, 198, 73,
                    55, 53, 59, 188, 26, 52, 74, 173, 179, 22, 213, 161, 71, 252, 125,
                ]),
                ListenPort(51820),
                Peers([peer_1, peer_2].to_vec()),
            ]
            .to_vec(),
        }
    }

    #[test]
    fn serialize_netlink_message() {
        let expected_payload: &[u8] = &[
            0x01, 0x01, 0x00, 0x00, 0x0c, 0x00, 0x02, 0x00, 0x77, 0x67, 0x2d, 0x74, 0x65, 0x73,
            0x74, 0x00, 0x24, 0x00, 0x03, 0x00, 0x38, 0x47, 0xf4, 0xad, 0x65, 0xdf, 0x55, 0x16,
            0xab, 0xaf, 0x0f, 0x27, 0x35, 0xb4, 0xc1, 0xc6, 0x49, 0x37, 0x35, 0x3b, 0xbc, 0x1a,
            0x34, 0x4a, 0xad, 0xb3, 0x16, 0xd5, 0xa1, 0x47, 0xfc, 0x7d, 0x06, 0x00, 0x06, 0x00,
            0x6c, 0xca, 0x00, 0x00, 0xcc, 0x00, 0x08, 0x80, 0x64, 0x00, 0x00, 0x80, 0x24, 0x00,
            0x01, 0x00, 0x20, 0xe0, 0x44, 0x05, 0x17, 0x88, 0x67, 0xe5, 0xce, 0x3b, 0x22, 0xe7,
            0xd7, 0x8b, 0xd6, 0xec, 0x50, 0x51, 0xbb, 0x07, 0x9a, 0xc5, 0xfb, 0x24, 0xab, 0x9c,
            0x30, 0x49, 0x91, 0x2f, 0x86, 0x36, 0x14, 0x00, 0x04, 0x00, 0x02, 0x00, 0x26, 0x45,
            0xc0, 0xa8, 0x28, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x20, 0x00, 0x09, 0x80, 0x1c, 0x00, 0x00, 0x80,
            0x06, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0xc0, 0xa8,
            0x27, 0x01, 0x05, 0x00, 0x03, 0x00, 0x20, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x80,
            0x24, 0x00, 0x01, 0x00, 0xf4, 0x1c, 0xce, 0x0c, 0x4f, 0x24, 0x58, 0xb7, 0xc2, 0x9d,
            0x36, 0x26, 0x36, 0xb7, 0x7f, 0x20, 0x8e, 0x18, 0xfb, 0x9e, 0xd9, 0x38, 0x0c, 0x92,
            0xd0, 0x15, 0x84, 0x9d, 0xa2, 0x44, 0x02, 0x2c, 0x14, 0x00, 0x04, 0x00, 0x02, 0x00,
            0x26, 0x45, 0xc0, 0xa8, 0x28, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x20, 0x00, 0x09, 0x80, 0x1c, 0x00,
            0x00, 0x80, 0x06, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0xc0, 0xa8, 0x27, 0x02, 0x05, 0x00, 0x03, 0x00, 0x20, 0x00, 0x00, 0x00,
        ];

        let mut message = sample_set_message();
        message.command = WG_CMD_SET_DEVICE;

        let mut payload_buffer = vec![0u8; message.buffer_len()];
        message.serialize(&mut payload_buffer);
        let header = NetlinkHeader {
            length: payload_buffer.len() as u32,
            message_type: 0,
            flags: 0,
            sequence_number: 0,
            port_number: 0,
        };
        let deserialized_device = DeviceMessage::deserialize(&header, &payload_buffer).unwrap();

        assert_eq!(message, deserialized_device);
        assert_eq!(payload_buffer, expected_payload);
    }
}
