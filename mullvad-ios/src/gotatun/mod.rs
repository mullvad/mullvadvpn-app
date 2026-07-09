pub mod connection_tracker;
pub mod ip_mux;
pub mod smoltcp_network;


/// Minimal IPv4 TCP packet with the given addresses, ports, and flags.
#[cfg(test)]
fn tcp_packet(
    src: impl Into<std::net::Ipv4Addr>,
    dst: impl Into<std::net::Ipv4Addr>,
    source_port: u16,
    destination_port: u16,
    flags: u8,
) -> Vec<u8> {
    use gotatun::packet::{IpNextProtocol, Ipv4, Ipv4Header, Tcp, TcpDataOffset, TcpHeader};
    use zerocopy::IntoBytes;

    let tcp = Tcp {
        header: TcpHeader {
            source_port: source_port.into(),
            destination_port: destination_port.into(),
            seq_num: 0.into(),
            ack_num: 0.into(),
            data_offset: TcpDataOffset::no_options(),
            flags: flags.into(),
            window: 0.into(),
            checksum: 0.into(),
            urgent_pointer: 0.into(),
        },
        options_and_payload: (),
    };
    Ipv4 {
        header: Ipv4Header::new(src.into(), dst.into(), IpNextProtocol::Tcp, tcp.as_bytes()),
        payload: tcp,
    }
    .as_bytes()
    .to_vec()
}
