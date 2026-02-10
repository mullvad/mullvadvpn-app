use clap::Parser;
use futures::FutureExt;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use mullvad_api::domain_fronting::server::Sessions;
use rustls_pki_types::{CertificateDer, pem::PemObject};
use std::{fs::File, io::BufReader, net::SocketAddr, path::{Path, PathBuf}, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[derive(Parser, Debug)]
#[clap(name = "domain_fronting_server")]
struct Args {
    /// Hostname for the server
    #[clap(short = 'H', long)]
    hostname: String,

    /// Path to certificate file (PEM format). If omitted, plain TCP is used.
    #[clap(short = 'c', long)]
    cert_path: Option<PathBuf>,

    /// Path to private key file (PEM format). Required if cert_path is set.
    #[clap(short = 'k', long)]
    key_path: Option<PathBuf>,

    /// Upstream socket address to forward CONNECT requests to
    #[clap(short = 'u', long)]
    upstream: SocketAddr,

    /// Port to listen on
    #[clap(short, long, default_value = "443")]
    port: u16,

    /// Session header key used to identify client sessions
    #[clap(short = 's', long)]
    session_header: String,
}

fn load_tls_config(
    cert_path: &std::path::Path,
    key_path: &std::path::Path,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Load certificate chain
    let cert_file = File::open(cert_path)?;
    let cert_chain =
        CertificateDer::pem_reader_iter(&mut std::io::BufReader::new(BufReader::new(cert_file)))
            .collect::<Result<Vec<_>, _>>()?;

    // Load private key
    let key = rustls_pki_types::PrivateKeyDer::from_pem_file(key_path)?;

    // Create server configuration
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .init();

    let args = Args::parse();
    let bind_addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;

    let tls_acceptor = match (&args.cert_path, &args.key_path) {
        (Some(cert_path), Some(key_path)) => {
            log::info!("Starting TLS domain fronting server on {}", bind_addr);
            log::info!("Cert path: {}", cert_path.display());
            log::info!("Key path: {}", key_path.display());
            let tls_config = load_tls_config(cert_path, key_path)?;
            Some(TlsAcceptor::from(Arc::new(tls_config)))
        }
        (None, None) => {
            log::info!("Starting plain TCP domain fronting server on {}", bind_addr);
            log::warn!("No TLS certificate provided - running without encryption");
            None
        }
        _ => {
            return Err("Both --cert-path and --key-path must be provided together".into());
        }
    };

    log::info!("Hostname: {}", args.hostname);
    log::info!("Upstream: {}", args.upstream);

    let listener = TcpListener::bind(bind_addr).await?;

    let sessions = Sessions::new(args.upstream, args.session_header);
    loop {
        let (stream, addr) = listener.accept().await?;

        log::debug!("Accepted connection from {addr}");

        let sessions = sessions.clone();
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            match tls_acceptor {
                Some(acceptor) => match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        serve_connection(TokioIo::new(tls_stream), sessions, addr).await;
                    }
                    Err(err) => {
                        log::error!("TLS handshake failed for {addr}: {err}");
                    }
                },
                None => {
                    serve_connection(TokioIo::new(stream), sessions, addr).await;
                }
            }
        });
    }
}

async fn serve_connection<S>(io: S, sessions: Sessions, addr: SocketAddr)
where
    S: hyper::rt::Read + hyper::rt::Write + Unpin + 'static,
{
    let service = service_fn(move |req| {
        sessions.clone().handle_request(req).map(Ok::<_, String>)
    });

    if let Err(err) = http1::Builder::new()
        .serve_connection(io, service)
        .with_upgrades()
        .await
    {
        log::error!("Error serving connection from {addr}: {err}");
    }
}
