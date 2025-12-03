use clap::Parser;
use http_body_util::{Empty, Full};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http2,
    service::service_fn,
    upgrade::Upgraded,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pemfile::{certs, private_key};
use std::{
    convert::Infallible, fs::File, io::BufReader, net::SocketAddr, path::PathBuf, sync::Arc,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};

#[derive(Parser, Debug)]
#[clap(name = "domain_fronting_server")]
struct Args {
    /// Hostname for the server
    #[clap(short, long)]
    hostname: String,

    /// Path to certificate file (PEM format)
    #[clap(short = 'c', long)]
    cert_path: PathBuf,

    /// Path to private key file (PEM format)
    #[clap(short = 'k', long)]
    key_path: PathBuf,

    /// Upstream socket address to forward CONNECT requests to
    #[clap(short = 'u', long)]
    upstream: SocketAddr,

    /// Port to listen on
    #[clap(short, long, default_value = "443")]
    port: u16,
}

async fn handle_connect(
    req: Request<Incoming>,
    upstream: SocketAddr,
) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("Log all requests {:?}", req);
    if req.method() == &Method::GET {
        println!("Responding to a GET: {:?}", req);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from_owner("hiya")))
            .unwrap());
    }
    let uri = req.uri();
    let _host = match uri.authority() {
        Some(auth) => auth.as_str(),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::new()))
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
        .body(Full::new(Bytes::new()))
        .unwrap())
}

async fn proxy_data(
    upgraded: Upgraded,
    upstream: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut upstream_stream = TcpStream::connect(upstream).await?;

    let mut client = TokioIo::new(upgraded);
    let _ = tokio::io::copy_bidirectional(&mut upstream_stream, &mut client).await?;
    Ok(())
}

fn load_tls_config(
    cert_path: &PathBuf,
    key_path: &PathBuf,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Load certificate chain
    let cert_file = File::open(cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain: Result<Vec<_>, _> = certs(&mut cert_reader).collect();
    let cert_chain = cert_chain?;

    // Load private key
    let key_file = File::open(key_path)?;
    let mut key_reader = BufReader::new(key_file);
    let key = private_key(&mut key_reader)?.ok_or("No private key found in key file")?;

    // Create server configuration
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let bind_addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;

    println!("Starting TLS domain fronting server on {}", bind_addr);
    println!("Hostname: {}", args.hostname);
    println!("Cert path: {}", args.cert_path.display());
    println!("Key path: {}", args.key_path.display());
    println!("Upstream: {}", args.upstream);

    // Load TLS configuration
    let tls_config = load_tls_config(&args.cert_path, &args.key_path)?;
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

    let listener = TcpListener::bind(bind_addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let upstream = args.upstream;
        let acceptor = tls_acceptor.clone();

        println!("Accepted connection from {}!", addr);

        tokio::spawn(async move {
            // Perform TLS handshake
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    println!("lmao what");
                    let io = TokioIo::new(tls_stream);
                    let service = service_fn(move |req| handle_connect(req, upstream));

                    if let Err(err) = http2::Builder::new(Executor{})
                        .serve_connection(io, service)
                        .await
                    {
                        eprintln!("Error serving connection from {}: {}", addr, err);
                    }
                }
                Err(err) => {
                    eprintln!("TLS handshake failed for {}: {}", addr, err);
                }
            }
        });
    }
}

#[derive(Clone)]
struct Executor {}
impl<F> hyper::rt::Executor<F> for Executor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}
