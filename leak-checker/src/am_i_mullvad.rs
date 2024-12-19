use anyhow::{anyhow, Context};
use futures::TryFutureExt;
use match_cfg::match_cfg;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;

use crate::{LeakInfo, LeakStatus};

#[derive(Clone, clap::Args)]
pub struct AmIMullvadOpt {
    /// Try to bind to a specific interface
    #[clap(short, long)]
    interface: Option<String>,
}

const AM_I_MULLVAD_URL: &str = "https://am.i.mullvad.net/json";

/// [try_run_leak_test], but on an error, assume we aren't leaking.
pub async fn run_leak_test(opt: &AmIMullvadOpt) -> LeakStatus {
    try_run_leak_test(opt)
        .await
        .inspect_err(|e| log::debug!("Leak test errored, assuming no leak. {e:?}"))
        .unwrap_or(LeakStatus::NoLeak)
}

/// Check if connected to Mullvad and print the result to stdout
pub async fn try_run_leak_test(opt: &AmIMullvadOpt) -> anyhow::Result<LeakStatus> {
    #[derive(Debug, Deserialize)]
    struct Response {
        ip: String,
        mullvad_exit_ip_hostname: Option<String>,
    }

    let mut client = Client::builder();

    if let Some(interface) = &opt.interface {
        client = bind_client_to_interface(client, interface)?;
    }

    let client = client.build().context("Failed to create HTTP client")?;
    let response: Response = client
        .get(AM_I_MULLVAD_URL)
        //.timeout(Duration::from_secs(opt.timeout))
        .send()
        .and_then(|r| r.json())
        .await
        .with_context(|| anyhow!("Failed to GET {AM_I_MULLVAD_URL}"))?;

    if let Some(server) = &response.mullvad_exit_ip_hostname {
        log::debug!(
            "You are connected to Mullvad (server {}). Your IP address is {}",
            server,
            response.ip
        );
        Ok(LeakStatus::NoLeak)
    } else {
        log::debug!(
            "You are not connected to Mullvad. Your IP address is {}",
            response.ip
        );
        Ok(LeakStatus::LeakDetected(LeakInfo::AmIMullvad {
            ip: response.ip.parse().context("Malformed IP")?,
        }))
    }
}

match_cfg! {
    #[cfg(target_os = "linux")] => {
        fn bind_client_to_interface(
            builder: ClientBuilder,
            interface: &str
        ) -> anyhow::Result<ClientBuilder> {
            log::debug!("Binding HTTP client to {interface}");
            Ok(builder.interface(interface))
        }
    }
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "android"))] => {
        fn bind_client_to_interface(
            builder: ClientBuilder,
            interface: &str
        ) -> anyhow::Result<ClientBuilder> {
            use crate::util::get_interface_ip;

            let ip = get_interface_ip(interface)?;

            log::debug!("Binding HTTP client to {ip} ({interface})");
            Ok(builder.local_address(ip))
        }
    }
}
