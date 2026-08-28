use clap::Parser;
use std::{io::stdin, time::Duration};
use tokio::runtime::Runtime;

use connection_checker::{
    cli::Opt,
    net::{send_ping, send_tcp, send_udp},
};

fn main() {
    let opt = Opt::parse();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    if opt.interactive {
        let stdin = stdin();
        for line in stdin.lines() {
            if line.is_err() {
                break;
            };
            test_connection(&opt, &runtime);
        }
    } else {
        test_connection(&opt, &runtime);
    }
}

fn test_connection(opt: &Opt, runtime: &Runtime) {
    if let Some(destination) = opt.leak {
        if opt.leak_tcp {
            let _ = send_tcp(opt, destination);
        }
        if opt.leak_udp {
            let _ = send_udp(opt, destination);
        }
        if opt.leak_icmp {
            let _ = send_ping(opt, destination.ip());
        }
    }
    am_i_mullvad(opt, runtime);
}

/// Check if connected to Mullvad and print the result to stdout
fn am_i_mullvad(opt: &Opt, runtime: &Runtime) {
    let ip_version = if opt.ipv6 {
        am_i_mullvad_client::IpVersion::V6
    } else {
        am_i_mullvad_client::IpVersion::V4
    };
    let result = runtime.block_on(am_i_mullvad_client::geoip_lookup(
        &opt.mullvad_host,
        ip_version,
        Duration::from_secs(opt.timeout),
    ));

    match result {
        Ok(response) => {
            if let Some(server) = &response.mullvad_exit_ip_hostname {
                println!(
                    "You are connected to Mullvad (server {}). Your IP address is {}",
                    server, response.ip
                );
            } else {
                println!(
                    "You are not connected to Mullvad. Your IP address is {}",
                    response.ip
                );
            }
        }
        Err(e) => {
            println!("Error: {e}");
        }
    }
}
