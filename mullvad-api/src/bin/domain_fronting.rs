use clap::Parser;
use http::{Method, Request};
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use mullvad_api::domain_fronting::DomainFronting;

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
    println!("front: {:?} host: {:?}", front, host);
    let domain_front = DomainFronting::new(front.clone(), host.clone());
    let connection = domain_front
        .proxy_config()
        .await
        .expect("Could not resolve {front}")
        .connect()
        .await
        .expect("Failed to connect to CDN");

    let (mut sender, conn) =
        hyper::client::conn::http1::handshake(TokioIo::new(connection)).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    // Build the request
    let req = Request::builder()
        .method(Method::GET)
        .header(hyper::header::HOST, host)
        .header(hyper::header::ACCEPT, "*/*")
        .body(Empty::<Bytes>::new())?;
    println!("request: {:?}", req);
    let response = sender.send_request(req).await?;

    println!("Response: {}", response.status());
    println!("Headers: {:#?}\n", response.headers());

    // Print the response to stdout
    let body: Bytes = response.collect().await?.to_bytes();
    tokio::io::copy(&mut body.as_ref(), &mut tokio::io::stdout()).await?;

    println!("\n\nDone!");
    Ok(())
}
#[cfg(feature = "domain-fronting")]
mod imp {}
