
unsigned long abstract_tun_size();

int fn abstract_tun_init_instance(
    params: *const IOSTunParams,
    object: *mut libc::c_void,
);

void abstract_tun_handle_tunnel_traffic(
    tun: *mut IOSTun,
    packet: *const u8,
    packet_size: usize,
);

void abstract_tun_handle_udp_packet(
    tun: *const IOSTun,
    packet: *const u8,
    packet_size: usize,
); 

void abstract_tun_handle_timer_event(tun: *const IOSTun);

void abstract_tun_drop(tun: *const IOSTun) 
