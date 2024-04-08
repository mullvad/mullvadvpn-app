use clap::Parser;
use eyre::{eyre, Context};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{io::stdin, time::Duration};

use connection_checker::{
    cli::Opt,
    net::{send_ping, send_tcp, send_udp},
};

fn main() -> eyre::Result<()> {
    let opt = Opt::parse();
    color_eyre::install()?;

    if opt.interactive {
        let stdin = stdin();
        for line in stdin.lines() {
            let _ = line.wrap_err("Failed to read from stdin")?;
            test_connection(&opt)?;
        }
    } else {
        test_connection(&opt)?;
    }

    Ok(())
}

fn test_connection(opt: &Opt) -> eyre::Result<bool> {
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
    am_i_mullvad(opt)
}

/// Check if connected to Mullvad and print the result to stdout
fn am_i_mullvad(opt: &Opt) -> eyre::Result<bool> {
    #[derive(Debug, Deserialize)]
    struct Response {
        ip: String,
        mullvad_exit_ip_hostname: Option<String>,
    }

    let url = "https://am.i.mullvad.net/json";

    let client = Client::new();
    let response: Response = client
        .get(url)
        .timeout(Duration::from_millis(opt.timeout))
        .send()
        .and_then(|r| r.json())
        .wrap_err_with(|| eyre!("Failed to GET {url}"))?;

    if let Some(server) = &response.mullvad_exit_ip_hostname {
        println!(
            "You are connected to Mullvad (server {}). Your IP address is {}",
            server, response.ip
        );
        Ok(true)
    } else {
        println!(
            "You are not connected to Mullvad. Your IP address is {}",
            response.ip
        );
        Ok(false)
    }
}
