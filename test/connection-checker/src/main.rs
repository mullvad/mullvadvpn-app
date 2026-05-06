use anyhow::{Context, anyhow};
use clap::Parser;
use serde::Deserialize;
use std::{io::stdin, sync::Arc, time::Duration};

use connection_checker::{
    cli::Opt,
    net::{send_ping, send_tcp, send_udp},
};

fn main() {
    let opt = Opt::parse();

    if opt.interactive {
        let stdin = stdin();
        for line in stdin.lines() {
            if line.is_err() {
                break;
            };
            test_connection(&opt);
        }
    } else {
        test_connection(&opt);
    }
}

fn test_connection(opt: &Opt) {
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
    am_i_mullvad(opt);
}

fn build_tls_config() -> rustls::ClientConfig {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let provider = rustls::crypto::CryptoProvider {
        kx_groups: vec![rustls::crypto::aws_lc_rs::kx_group::X25519MLKEM768],
        ..rustls::crypto::aws_lc_rs::default_provider()
    };
    rustls::ClientConfig::builder_with_provider(Arc::new(provider))
        .with_safe_default_protocol_versions()
        .expect("aws-lc-rs should support default TLS versions")
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

/// Check if connected to Mullvad and print the result to stdout
fn am_i_mullvad(opt: &Opt) {
    #[derive(Debug, Deserialize)]
    struct Response {
        ip: String,
        mullvad_exit_ip_hostname: Option<String>,
    }

    let url = &opt.url;

    let client = reqwest::blocking::Client::builder()
        .use_preconfigured_tls(build_tls_config())
        .build()
        .expect("Failed to build HTTP client");
    let result: Result<Response, _> = client
        .get(url)
        .timeout(Duration::from_secs(opt.timeout))
        .send()
        .and_then(|r| r.json())
        .with_context(|| anyhow!("Failed to GET {url}"));

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
