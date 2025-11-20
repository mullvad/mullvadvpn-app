#[cfg(target_os = "macos")]
mod imp {
    use clap::Parser;
    use http::{Method, Request};
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::rt::TokioIo;
    use mullvad_api::domain_fronting::DomainFronting;
    use tokio::io::{self, AsyncWriteExt as _};

    #[derive(Parser, Debug)]
    pub struct Arguments {
        /// The domain used to hide the actual destination.
        #[arg(long)]
        front: String,

        /// The host being reached via `front`.
        #[arg(long)]
        host: String,
    }

    pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .parse_default_env()
            .init();

        let Arguments { front, host } = Arguments::parse();
        println!("front: {:?} host: {:?}", front, host);
        let domain_front = DomainFronting::new(front.clone());
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
        });

        // Build the request
        let req = Request::builder()
            .method(Method::GET)
            .header(hyper::header::HOST, host)
            .header(hyper::header::ACCEPT, "*/*")
            .body(Empty::<Bytes>::new())?;
        println!("request: {:?}", req);
        let mut res = sender.send_request(req).await?;

        println!("Response: {}", res.status());
        println!("Headers: {:#?}\n", res.headers());

        // Stream the body, writing each chunk to stdout as we get it
        // (instead of buffering and printing at the end).
        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                io::stdout().write_all(chunk).await?;
            }
        }

        println!("\n\nDone!");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    imp::main().await
}

#[cfg(not(target_os = "macos"))]
pub mod imp {
    pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}
