use eyre::{eyre, Context};
use reqwest::blocking::get;
use serde::Deserialize;
use std::process;

#[derive(Debug, Deserialize)]
struct Response {
    ip: String,
    mullvad_exit_ip_hostname: Option<String>,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let url = "https://am.i.mullvad.net/json";
    let response: Response = get(url)
        .and_then(|r| r.json())
        .wrap_err_with(|| eyre!("Failed to GET {url}"))?;

    if let Some(server) = &response.mullvad_exit_ip_hostname {
        println!(
            "You are connected to Mullvad (server {}). Your IP address is {}",
            server, response.ip
        );
        Ok(())
    } else {
        println!(
            "You are not connected to Mullvad. Your IP address is {}",
            response.ip
        );
        process::exit(1)
    }
}
