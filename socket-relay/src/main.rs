extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate tokio_core;

extern crate socket_relay;

use std::env;
use std::thread;
use std::time::Duration;

use tokio_core::reactor::Core;

error_chain!{}

quick_main!(run);
fn run() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to init logging")?;

    let listen_addr = env::args()
        .nth(1)
        .expect("Listen address as first argument")
        .parse()
        .expect("Invalid listen address format");
    let destination = env::args()
        .nth(2)
        .expect("Relay destination address as second argument")
        .parse()
        .expect("Invalid destination address format");
    let forward_bind_ip = env::args()
        .nth(3)
        .unwrap_or(String::from("0.0.0.0"))
        .parse()
        .unwrap();

    let mut core = Core::new().chain_err(|| "Unable to create Tokio core")?;
    let handle = core.handle();

    let relay = socket_relay::udp::Relay::new(listen_addr, forward_bind_ip, destination, handle)
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
