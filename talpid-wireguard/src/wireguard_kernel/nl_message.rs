use super::parsers;
use byteorder::{ByteOrder, NativeEndian};
use netlink_packet_core::{
    NetlinkDeserializable, NetlinkHeader, NetlinkPayload, NetlinkSerializable,
};
use netlink_packet_utils::{
    nla::{Nla, NlaBuffer, NlasIterator},
    traits::{Emitable, Parseable},
    DecodeError,
};
use std::{ffi::CString, io::Write, mem};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Family name too long")]
    FamilyNameTooLong,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetlinkControlMessage {
    cmd: u8,
    version: u8,
    pub nlas: Vec<ControlNla>,
}

impl NetlinkControlMessage {
    pub fn get_netlink_family_id(name: CString) -> Result<Self, Error> {
        if name.as_bytes_with_nul().len() > (libc::GENL_NAMSIZ as usize) {
            return Err(Error::FamilyNameTooLong);
        }
        Ok(Self {
            nlas: vec![ControlNla::FamilyName(name)],
            cmd: libc::CTRL_CMD_GETFAMILY as u8,
            version: 1,
        })
    }
}

impl NetlinkSerializable for NetlinkControlMessage {
    fn message_type(&self) -> u16 {
        libc::GENL_ID_CTRL as u16
    }

    fn buffer_len(&self) -> usize {
        mem::size_of::<libc::genlmsghdr>() + self.nlas.as_slice().buffer_len()
    }

    fn serialize(&self, mut buffer: &mut [u8]) {
        let _ = buffer.write(&[self.cmd, self.version, 0u8, 0u8]).unwrap();
        self.nlas.as_slice().emit(buffer);
    }
}

impl From<NetlinkControlMessage> for NetlinkPayload<NetlinkControlMessage> {
    fn from(msg: NetlinkControlMessage) -> Self {
        NetlinkPayload::InnerMessage(msg)
    }
}

impl NetlinkDeserializable for NetlinkControlMessage {
    type Error = DecodeError;
    fn deserialize(
        _header: &NetlinkHeader,
        payload: &[u8],
    ) -> Result<NetlinkControlMessage, Self::Error> {
        // skip the genlmsghdr
        let (cmd, version) = parsers::parse_genlmsghdr(payload)?;
        let nla_buffer = &payload[mem::size_of::<libc::genlmsghdr>()..];
        let nlas = NlasIterator::new(nla_buffer)
            .map(|buffer| ControlNla::parse(&buffer?))
            .collect::<Result<Vec<_>, DecodeError>>()?;

        Ok(NetlinkControlMessage { nlas, cmd, version })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ControlNla {
    FamilyName(CString),
    FamilyId(u16),
    Unknown(u16, Vec<u8>),
}

impl Nla for ControlNla {
    fn value_len(&self) -> usize {
        use ControlNla::*;
        match self {
            FamilyName(name) => name.as_bytes_with_nul().len(),
            FamilyId(_id) => 2,
            Unknown(_, buffer) => buffer.len(),
        }
    }

    fn kind(&self) -> u16 {
        use ControlNla::*;
        match self {
            FamilyName(_) => libc::CTRL_ATTR_FAMILY_NAME as u16,
            FamilyId(_) => libc::CTRL_ATTR_FAMILY_ID as u16,
            Unknown(kind, _) => *kind,
        }
    }

    fn emit_value(&self, mut buffer: &mut [u8]) {
        use ControlNla::*;
        match self {
            FamilyName(name) => {
                let _ = buffer.write(name.as_bytes()).unwrap();
            }
            FamilyId(id) => {
                NativeEndian::write_u16(buffer, *id);
            }

            Unknown(_, value) => {
                let _ = buffer.write(value).unwrap();
            }
        }
    }
}

impl<'a, T: AsRef<[u8]> + 'a + ?Sized + std::fmt::Debug> Parseable<NlaBuffer<&'a T>>
    for ControlNla
{
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        let nla = match buf.kind() as i32 {
            libc::CTRL_ATTR_FAMILY_NAME => {
                ControlNla::FamilyName(parsers::parse_cstring(buf.value())?)
            }
            libc::CTRL_ATTR_FAMILY_ID => ControlNla::FamilyId(parsers::parse_u16(buf.value())?),
            _unknown_kind => ControlNla::Unknown(buf.kind(), buf.value().to_vec()),
        };
        Ok(nla)
    }
}
