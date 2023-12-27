use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Once,
};

use crate::{Config, IoBuffer, PeerConfig, WgInstance};

pub mod data;
use data::SwiftDataArray;
mod udp_session;

const INIT_LOGGING: Once = Once::new();

pub struct IOSTun {
    wg: super::WgInstance,
}

#[repr(C)]
pub struct IOSTunParams {
    private_key: [u8; 32],
    peer_key: [u8; 32],
    peer_addr_version: u8,
    peer_addr_bytes: [u8; 16],
    peer_port: u16,
}

impl IOSTunParams {
    fn peer_addr(&self) -> Option<IpAddr> {
        match self.peer_addr_version as i32 {
            libc::AF_INET => Some(
                Ipv4Addr::new(
                    self.peer_addr_bytes[0],
                    self.peer_addr_bytes[1],
                    self.peer_addr_bytes[2],
                    self.peer_addr_bytes[3],
                )
                .into(),
            ),
            libc::AF_INET6 => Some(Ipv6Addr::from(self.peer_addr_bytes).into()),
            _other => None,
        }
    }
}

pub struct IOSUdpSender {
    // current assumption is that we only send data to a single endpoint.
    v4_buffer: SwiftDataArray,
    v6_buffer: SwiftDataArray,
}

impl IOSUdpSender {
    fn new() -> Self {
        Self {
            v4_buffer: SwiftDataArray::new(),
            v6_buffer: SwiftDataArray::new(),
        }
    }

    pub fn drain_v4_buffer(&mut self) -> SwiftDataArray {
        self.v4_buffer.drain()
    }

    pub fn drain_v6_buffer(&mut self) -> SwiftDataArray {
        self.v6_buffer.drain()
    }
}

#[no_mangle]
pub extern "C" fn abstract_tun_size() -> usize {
    std::mem::size_of::<IOSTun>()
}

#[no_mangle]
pub extern "C" fn abstract_tun_init_instance(params: *const IOSTunParams) -> *mut IOSTun {
    // INIT_LOGGING.call_once(|| {
    //     let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.ShadowSocks")
    //         .level_filter(log::LevelFilter::Error)
    //         .init();
    // });

    let params = unsafe { &*params };
    let peer_addr = match params.peer_addr() {
        Some(addr) => addr,
        None => {
            return std::ptr::null_mut();
        }
    };

    let config = Config {
        // TODO: Use real address
        #[cfg(not(target_os = "ios"))]
        address: Ipv4Addr::UNSPECIFIED,
        private_key: params.private_key,
        peers: vec![PeerConfig {
            endpoint: SocketAddr::new(peer_addr, params.peer_port),
            pub_key: params.peer_key,
        }],
    };

    // SAFETY: TODO
    let ptr = Box::into_raw(Box::new(IOSTun {
        wg: WgInstance::new(config),
    }));

    ptr
}

#[repr(C)]
pub struct IOOutput {
    pub udp_v4_output: *mut libc::c_void,
    pub udp_v6_output: *mut libc::c_void,
    pub tun_v4_output: *mut libc::c_void,
    pub tun_v6_output: *mut libc::c_void,
}


#[no_mangle]
pub extern "C" fn abstract_tun_handle_host_traffic(
    tun: *mut IOSTun,
    packets: *mut libc::c_void,
) -> IOOutput {
    let tun: &mut IOSTun = unsafe { &mut *(tun) };
    let mut packets = unsafe { SwiftDataArray::from_ptr(packets as *mut _) };
    let mut output_buffer = IoBuffer::new();

    for mut packet in packets.iter() {
        tun.wg
            .handle_host_traffic(packet.as_mut(), &mut output_buffer);
    }

    output_buffer.to_output()
}

#[no_mangle]
pub extern "C" fn abstract_tun_handle_tunnel_traffic(
    tun: *mut IOSTun,
    packets: *mut libc::c_void,
) -> IOOutput {
    let tun: &mut IOSTun = unsafe { &mut *(tun as *mut _) };
    let mut packets = unsafe { SwiftDataArray::from_ptr(packets as *mut _) };
    let mut output_buffer = IoBuffer::new();

    for mut packet in packets.iter() {
        tun.wg
            .handle_tunnel_traffic(packet.as_mut(), &mut output_buffer);
    }

    output_buffer.to_output()
}

#[no_mangle]
pub extern "C" fn abstract_tun_handle_timer_event(tun: *mut IOSTun) -> IOOutput {
    let tun: &mut IOSTun = unsafe { &mut *(tun as *mut _) };
    let mut output_buffer = IoBuffer::new();
    tun.wg.handle_timer_tick(&mut output_buffer);
    output_buffer.to_output()
}

#[no_mangle]
pub extern "C" fn abstract_tun_drop(tun: *mut IOSTun) {
    if tun.is_null() {
        return;
    }
    let tun: Box<IOSTun> = unsafe { Box::from_raw(tun) };
    std::mem::drop(tun);
}

#[no_mangle]
pub extern "C" fn test_vec(_idx: i64) {
    // let mut vec = SwiftDataArray::new();
    // for i in 0..1024 {
    //     let buf = vec![0u8; 2048];
    //     vec.append(&buf);
    // }
    // std::mem::drop(vec);
}

#[no_mangle]
pub extern "C" fn test_mallocsing() -> IOOutput {
    let mut output_buffer = IoBuffer::new();
    for _ in 0..1024 {
        let mut buf = [0u8; 1500];
        for (idx, i) in buf.iter_mut().enumerate() {
            *i = i.wrapping_add(idx as u8);
        }
        output_buffer.tun_v4_output.append(buf.as_slice());
    }

    output_buffer.to_output()
}
