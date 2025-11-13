use std::ffi::c_uchar;

use zerocopy::{FromBytes, Immutable, IntoBytes};

#[derive(Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct NlMsgHdr {
    pub nlmsg_len: u32,
    pub nlmsg_type: u16,
    pub nlmsg_flags: u16,
    pub nlmsg_seq: u32,
    pub nlmsg_pid: u32,
}

#[derive(Immutable, FromBytes, IntoBytes)]
#[repr(C)]
struct NlMsg {
    pub header: NlMsgHdr,
    pub payload: [u8],
}

#[derive(Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct RtGenMsg {
    // unsigned char rtgen_family
    pub rtgen_family: c_uchar,
}

#[derive(Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct NlAttr {
    pub nla_len: u16,
    pub nla_type: u16,
}

// RTM_GETNSID
// NlMsgHdr, /*padding?*/, RtGenMsg, /*padding?*/, NlAttr, /*padding?*/
