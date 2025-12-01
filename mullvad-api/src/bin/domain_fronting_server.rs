use clap::Parser;
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[clap(name = "domain_fronting_server")]
struct Args {
    /// Hostname for the server
    #[clap(short, long)]
    hostname: String,

    /// Path to root certificate file
    #[clap(short = 'c', long)]
    cert_path: PathBuf,

    /// Upstream socket address to forward CONNECT requests to
    #[clap(short = 'u', long)]
    upstream: SocketAddr,

    /// Port to listen on
    #[clap(short, long, default_value = "80")]
    port: u16,
}

async fn handle_connect(
    req: Request<Incoming>,
    upstream: SocketAddr,
) -> Result<Response<Empty<Bytes>>, Infallible> {
    if req.method() != Method::CONNECT {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Empty::new())
            .unwrap());
    }

    let uri = req.uri();
    let _host = match uri.authority() {
        Some(auth) => auth.as_str(),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Empty::new())
                .unwrap());
        }
    };

    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                if let Err(e) = proxy_data(upgraded, upstream).await {
                    eprintln!("Failed to proxy data: {}", e);
                }
            }
            Err(e) => eprintln!("Upgrade error: {}", e),
        }
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Empty::new())
        .unwrap())
}

async fn proxy_data(upgraded: Upgraded, upstream: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let upstream_stream = TcpStream::connect(upstream).await?;

    // Split the TCP stream for bidirectional usage
    let (upstream_read, upstream_write) = upstream_stream.into_split();
    
    // Use TokioIo wrapper for the upgraded connection
    let client = TokioIo::new(upgraded);
    let (client_read, client_write) = tokio::io::split(client);

    // Spawn two separate futures to forward data bidirectionally
    let client_to_upstream = copy_data(client_read, upstream_write);
    let upstream_to_client = copy_data(upstream_read, client_write);

    // Run both copy operations concurrently
    tokio::try_join!(client_to_upstream, upstream_to_client)?;
    
    Ok(())
}

async fn copy_data<R, W>(mut reader: R, mut writer: W) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; 8192];
    let mut total = 0u64;
    
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n]).await?;
        total += n as u64;
    }
    
    Ok(total)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let bind_addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;

    println!("Starting domain fronting server on {}", bind_addr);
    println!("Hostname: {}", args.hostname);
    println!("Cert path: {}", args.cert_path.display());
    println!("Upstream: {}", args.upstream);

    let listener = TcpListener::bind(bind_addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let upstream = args.upstream;

        println!("Accepted connection from {}", addr);

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| handle_connect(req, upstream));
            
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                eprintln!("Error serving connection from {}: {}", addr, err);
            }
        });
    }
}
