extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate tokio_core;

extern crate socket_relay;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::time::Duration;

use tokio_core::reactor::Core;

error_chain!{}

quick_main!(run);
fn run() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to init logging")?;

    let listen_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let listen_port = 53;
    let listen_addr = SocketAddr::new(listen_ip, listen_port);

    let forward_bind_ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    let forward_ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let forward_port = 53;
    let forward_addr = SocketAddr::new(forward_ip, forward_port);


    let mut core = Core::new().chain_err(|| "Unable to create Tokio core")?;
    let handle = core.handle();

    let relay = socket_relay::udp::Relay::new(listen_addr, forward_bind_ip, forward_addr, handle)
        .chain_err(|| "Unable to init forwarder")?;
    println!("Forwarder listening on {}", relay.listen_addr());

    let close_handle = relay.close_handle();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(20));
        println!("Closing relay");
        close_handle.close();
    });

    let result = core.run(relay);
    println!("result: {:?}", result);
    Ok(())
}
