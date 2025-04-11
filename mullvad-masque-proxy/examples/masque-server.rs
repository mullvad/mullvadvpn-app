use clap::Parser;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use std::{
    fs,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Parser, Debug)]
pub struct ServerArgs {
    /// Bind address
    #[arg(long, short = 'b', default_value = "0.0.0.0:0")]
    bind_addr: SocketAddr,

    /// Path to cert
    #[arg(long, short = 'c')]
    cert_path: PathBuf,

    /// Path to key
    #[arg(long, short = 'k')]
    key_path: PathBuf,

    /// Allowed IPs
    #[arg(long = "allowed-ip", short = 'a', required = false)]
    allowed_ips: Vec<IpAddr>,

    /// Maximum packet size
    #[arg(long, short = 'm', default_value = "1700")]
    mtu: u16,
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let args = ServerArgs::parse();
    let _keylog = rustls::KeyLogFile::new();

    let tls_config = load_server_config(&args.key_path, &args.cert_path).unwrap();

    let server = mullvad_masque_proxy::server::Server::bind(
        args.bind_addr,
        args.allowed_ips.iter().cloned().collect(),
        tls_config.into(),
        args.mtu,
    )
    .expect("Failed to initialize server");
    log::debug!("Listening on {}", args.bind_addr);
    server.run().await.expect("Server failed.")
}

fn load_server_config(
    key_path: &Path,
    cert_path: &Path,
) -> Result<rustls::ServerConfig, Box<dyn std::error::Error>> {
    let key = fs::read(key_path)?;
    let key = if key_path.extension().is_some_and(|x| x == "der") {
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key))
    } else {
        rustls_pemfile::private_key(&mut &*key)?.expect("Expected PEM file to contain private key")
    };
    let cert_chain = fs::read(cert_path)?;
    let cert_chain = if cert_path.extension().is_some_and(|x| x == "der") {
        vec![CertificateDer::from(cert_chain)]
    } else {
        rustls_pemfile::certs(&mut &*cert_chain).collect::<Result<_, _>>()?
    };

    let mut tls_config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])?
    .with_no_client_auth()
    .with_single_cert(cert_chain, key)?;

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![b"h3".into()];

    Ok(tls_config)
}
