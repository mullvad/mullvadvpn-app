extern crate env_logger;
extern crate socket_relay;
extern crate tokio_core;

use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use socket_relay::udp::Relay;
use tokio_core::reactor::Core;

#[test]
fn test() {
    env_logger::init().unwrap();

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut buffer = [0; 100];

    let relay_listen_addr = spawn_relay(socket.local_addr().unwrap());

    let test_data = [9, 88, 5, 2];
    socket.send_to(&test_data, relay_listen_addr).unwrap();
    let (len1, relay_src1) = socket.recv_from(&mut buffer).unwrap();
    assert_eq!(&buffer[..len1], &test_data);

    let reply_test_data = [1, 2, 6, 100];
    socket.send_to(&reply_test_data, relay_src1).unwrap();
    let (len2, relay_src2) = socket.recv_from(&mut buffer).unwrap();
    assert_eq!(relay_src2, relay_listen_addr);
    assert_eq!(&buffer[..len2], &reply_test_data);
}

fn spawn_relay(destination_addr: SocketAddr) -> SocketAddr {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let relay = Relay::new(
            "127.0.0.1:0".parse().unwrap(),
            "127.0.0.1".parse().unwrap(),
            destination_addr,
            handle,
        ).unwrap();
        println!("Relay listening on {}", relay.listen_addr());
        tx.send(relay.listen_addr()).unwrap();
        let _ = core.run(relay);
        println!("Relay exiting")
    });
    rx.recv().unwrap()
}
