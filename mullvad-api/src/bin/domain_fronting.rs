use std::{io::Write, sync::Arc};

use clap::Parser;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use mullvad_api::{
    domain_fronting::DomainFronting,
    https_client_with_sni::HttpsConnectorWithSni,
    proxy::{ApiConnectionMode, ProxyConfig},
};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[derive(Parser, Debug)]
pub struct Arguments {
    /// The domain used to hide the actual destination.
    #[arg(long)]
    front: String,

    /// The host being reached via `front`.
    #[arg(long)]
    host: String,

    /// Session header key used to identify client sessions
    #[clap(short = 's', long)]
    session_header: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .init();

    let Arguments {
        front,
        host,
        session_header,
    } = Arguments::parse();

    let df = DomainFronting::new(front, host, session_header);

    let proxy_config = df.proxy_config().await.unwrap();

    let (connector, connector_handle) = HttpsConnectorWithSni::new(
        Arc::new(mullvad_api::DefaultDnsResolver),
        #[cfg(feature = "api-override")]
        false,
    );

    connector_handle.set_connection_mode(ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(
        proxy_config,
    )));

    let client: Client<_, Full<Bytes>> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(connector);

    let response = client
        .get("https://api.mullvad.net/app/v1/relays".try_into().unwrap())
        .await
        .unwrap();
    log::info!("Response status: {}", response.status());
    log::debug!("Response headers: {:?}", response.headers());
    let body = response
        .collect()
        .await
        .expect("failed to fetch response body")
        .to_bytes();
    let _ = std::io::stdout().write(&body);
    let _ = std::io::stdout().write(b"\n");
    Ok(())
}
