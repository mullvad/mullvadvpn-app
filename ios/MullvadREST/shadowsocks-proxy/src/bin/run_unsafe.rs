use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

fn main() {
    let socketaddr = SocketAddr::from_str("185.65.135.117:443").unwrap();
    let password = "mullvad";
    let cipher = "aes-256-gcm";

    let cipher_ptr = cipher.as_ptr();
    let cipher_size = cipher.as_bytes().len();

    let password_ptr = password.as_ptr();
    let password_size = password.as_bytes().len();

    let addr = Ipv4Addr::from_str("185.65.135.117").unwrap();
    let addr_bytes = addr.octets();
    let addr_ptr = addr_bytes.as_ptr();

    let mut ctx = shadowsocks_proxy::ProxyHandle {
        port: 0,
        context: std::ptr::null_mut(),
    };

    let retval = shadowsocks_proxy::start_shadowsocks_proxy(
        addr_ptr,
        addr_bytes.len(),
        socketaddr.port(),
        password_ptr,
        password_size,
        cipher_ptr,
        cipher_size,
        &mut ctx as *mut _,
    );
    if retval != 0 {
        println!("Failed to start proxy - {retval}");
        return;
    }

    println!("Running proxy on port {}", ctx.port);
    let _ = std::io::stdin().read_line(&mut String::new());
    println!("Stopping proxy");
    let retval = shadowsocks_proxy::stop_shadowsocks_proxy(&mut ctx as *mut _);
    if retval != 0 {
        println!("Failed to stop proxy");
    }
    println!("Done");
}
