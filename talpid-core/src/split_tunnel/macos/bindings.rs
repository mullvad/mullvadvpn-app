/* automatically generated by rust-bindgen 0.70.1 */

pub const PTH_FLAG_DIR_OUT: u32 = 2;
pub type __int32_t = ::std::os::raw::c_int;
pub type __darwin_pid_t = __int32_t;
pub type __darwin_uuid_t = [::std::os::raw::c_uchar; 16usize];
pub type pid_t = __darwin_pid_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timeval32 {
    pub tv_sec: __int32_t,
    pub tv_usec: __int32_t,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of timeval32"][::std::mem::size_of::<timeval32>() - 8usize];
    ["Alignment of timeval32"][::std::mem::align_of::<timeval32>() - 4usize];
    ["Offset of field: timeval32::tv_sec"][::std::mem::offset_of!(timeval32, tv_sec) - 0usize];
    ["Offset of field: timeval32::tv_usec"][::std::mem::offset_of!(timeval32, tv_usec) - 4usize];
};
pub type uuid_t = __darwin_uuid_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pktap_header {
    pub pth_length: u32,
    pub pth_type_next: u32,
    pub pth_dlt: u32,
    pub pth_ifname: [::std::os::raw::c_char; 24usize],
    pub pth_flags: u32,
    pub pth_protocol_family: u32,
    pub pth_frame_pre_length: u32,
    pub pth_frame_post_length: u32,
    pub pth_pid: pid_t,
    pub pth_comm: [::std::os::raw::c_char; 17usize],
    pub pth_svc: u32,
    pub pth_iftype: u16,
    pub pth_ifunit: u16,
    pub pth_epid: pid_t,
    pub pth_ecomm: [::std::os::raw::c_char; 17usize],
    pub pth_flowid: u32,
    pub pth_ipproto: u32,
    pub pth_tstamp: timeval32,
    pub pth_uuid: uuid_t,
    pub pth_euuid: uuid_t,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pktap_header"][::std::mem::size_of::<pktap_header>() - 156usize];
    ["Alignment of pktap_header"][::std::mem::align_of::<pktap_header>() - 4usize];
    ["Offset of field: pktap_header::pth_length"]
        [::std::mem::offset_of!(pktap_header, pth_length) - 0usize];
    ["Offset of field: pktap_header::pth_type_next"]
        [::std::mem::offset_of!(pktap_header, pth_type_next) - 4usize];
    ["Offset of field: pktap_header::pth_dlt"]
        [::std::mem::offset_of!(pktap_header, pth_dlt) - 8usize];
    ["Offset of field: pktap_header::pth_ifname"]
        [::std::mem::offset_of!(pktap_header, pth_ifname) - 12usize];
    ["Offset of field: pktap_header::pth_flags"]
        [::std::mem::offset_of!(pktap_header, pth_flags) - 36usize];
    ["Offset of field: pktap_header::pth_protocol_family"]
        [::std::mem::offset_of!(pktap_header, pth_protocol_family) - 40usize];
    ["Offset of field: pktap_header::pth_frame_pre_length"]
        [::std::mem::offset_of!(pktap_header, pth_frame_pre_length) - 44usize];
    ["Offset of field: pktap_header::pth_frame_post_length"]
        [::std::mem::offset_of!(pktap_header, pth_frame_post_length) - 48usize];
    ["Offset of field: pktap_header::pth_pid"]
        [::std::mem::offset_of!(pktap_header, pth_pid) - 52usize];
    ["Offset of field: pktap_header::pth_comm"]
        [::std::mem::offset_of!(pktap_header, pth_comm) - 56usize];
    ["Offset of field: pktap_header::pth_svc"]
        [::std::mem::offset_of!(pktap_header, pth_svc) - 76usize];
    ["Offset of field: pktap_header::pth_iftype"]
        [::std::mem::offset_of!(pktap_header, pth_iftype) - 80usize];
    ["Offset of field: pktap_header::pth_ifunit"]
        [::std::mem::offset_of!(pktap_header, pth_ifunit) - 82usize];
    ["Offset of field: pktap_header::pth_epid"]
        [::std::mem::offset_of!(pktap_header, pth_epid) - 84usize];
    ["Offset of field: pktap_header::pth_ecomm"]
        [::std::mem::offset_of!(pktap_header, pth_ecomm) - 88usize];
    ["Offset of field: pktap_header::pth_flowid"]
        [::std::mem::offset_of!(pktap_header, pth_flowid) - 108usize];
    ["Offset of field: pktap_header::pth_ipproto"]
        [::std::mem::offset_of!(pktap_header, pth_ipproto) - 112usize];
    ["Offset of field: pktap_header::pth_tstamp"]
        [::std::mem::offset_of!(pktap_header, pth_tstamp) - 116usize];
    ["Offset of field: pktap_header::pth_uuid"]
        [::std::mem::offset_of!(pktap_header, pth_uuid) - 124usize];
    ["Offset of field: pktap_header::pth_euuid"]
        [::std::mem::offset_of!(pktap_header, pth_euuid) - 140usize];
};
