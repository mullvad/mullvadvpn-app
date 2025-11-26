#[tokio::main]
async fn main() -> anyhow::Result<()> {
    imp::main().await
}

#[cfg(not(feature = "domain-fronting"))]
pub mod imp {
    pub async fn main() -> anyhow::Result<()> {
        unimplemented!(
            "cargo run -p mullvad-api --features domain-fronting --bin domain_fronting -- --front <FRONT_DOMAIN> --host <HOST_DOMAIN>"
        )
    }
}

#[cfg(feature = "domain-fronting")]
mod imp {
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

    pub async fn main() -> anyhow::Result<()> {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .parse_default_env()
            .init();

        let Arguments { front, host } = Arguments::parse();
        println!("front: {:?} host: {:?}", front, host);
        let domain_front = DomainFronting::new(front.clone());
        let tls_stream = domain_front
            .connect()
            .await
            .expect("Could not resolve {front}");

        let io = TokioIo::new(tls_stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

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
        let res = sender.send_request(req).await?;

        println!("Response: {}", res.status());
        println!("Headers: {:#?}\n", res.headers());

        // Print the response to stdout
        let body = res.collect().await?.to_bytes();
        tokio::io::copy(&mut body.as_ref(), &mut tokio::io::stdout()).await?;

        println!("\n\nDone!");
        Ok(())
    }
}
