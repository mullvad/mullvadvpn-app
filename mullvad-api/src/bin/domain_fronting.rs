use clap::Parser;
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
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let Arguments { front, host } = Arguments::parse();
    // Do we want the HTTP2 feature for hyper ?, no we don't
    println!("front: {:?} host: {:?}", front, host);
    // Use this to resolve DNS

    let domain_front = DomainFronting::new(host, front);

    let tls_stream = domain_front
        .try_connect()
        .await
        .expect("Could not resolve {front}");

    let io = TokioIo::new(tls_stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    })
    .await;

    Ok(())
}
