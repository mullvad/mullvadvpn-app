use std::{io::Write, sync::Arc};

use clap::Parser;
use http::{Method, Request};
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::body::Bytes;
use hyper_util::{client::legacy::Client, rt::TokioIo};
use mullvad_api::{
    domain_fronting::DomainFronting,
    https_client_with_sni::HttpsConnectorWithSni,
    proxy::{ApiConnectionMode, ProxyConfig},
};

#[derive(Parser, Debug)]
pub struct Arguments {
    /// The domain used to hide the actual destination.
    #[arg(long)]
    front: String,

    /// The host being reached via `front`.
    #[arg(long)]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let Arguments { front, host } = Arguments::parse();

    let df = DomainFronting::new(front, host);

    let proxy_config = df.proxy_config().await.unwrap();

    let (connector, connector_handle) = HttpsConnectorWithSni::new(
        Arc::new(mullvad_api::DefaultDnsResolver),
        #[cfg(any(feature = "api-override", test))]
        false,
    );

    connector_handle.set_connection_mode(ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(
        proxy_config,
    )));

    let client: Client<_, Full<Bytes>> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(connector);

    let response = client
        .get(
            "https://api.mullvad.net/app/v1/relays"
                .try_into()
                .unwrap(),
        )
        .await
        .unwrap();
    println!("response: {:?}", response);
    let body = response
        .collect()
        .await
        .expect("failed to fetch response body")
        .to_bytes();
    let _ = std::io::stdout().write(&body);
    println!("");
    Ok(())
}

#[cfg(feature = "domain-fronting")]
mod imp {}
