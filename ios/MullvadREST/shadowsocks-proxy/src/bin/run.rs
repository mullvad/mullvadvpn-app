use std::{net::SocketAddr, str::FromStr};

fn main() {
    let socketaddr = SocketAddr::from_str("185.65.135.117:443").unwrap();
    let password = "mullvad";
    let cipher = "aes-256-gcm";

    let forward_address = SocketAddr::from_str("45.83.223.196:443").unwrap();

    let (port, handle) =
        shadowsocks_proxy::run_forwarding_proxy(forward_address, socketaddr, password, cipher)
            .expect("failed to start SOCKS proxy");

    println!("Running proxy on port {port}");

    let _ = std::io::stdin().read_line(&mut String::new());
    println!("Stopping proxy");
    handle.stop();
    println!("Done");
}
