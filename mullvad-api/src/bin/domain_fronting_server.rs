use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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

async fn handle_client(mut client: TcpStream, upstream: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(&mut client);
    let mut request_line = String::new();

    // Read the first line (HTTP request line)
    reader.read_line(&mut request_line).await?;

    if request_line.starts_with("CONNECT ") {
        // Parse the CONNECT request
        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let _target = parts[1]; // The target host:port (not used in this simple implementation)

            // Read and discard headers until empty line
            let mut line = String::new();
            loop {
                line.clear();
                reader.read_line(&mut line).await?;
                if line.trim().is_empty() {
                    break;
                }
            }

            // Try to connect to upstream
            match TcpStream::connect(upstream).await {
                Ok(upstream_stream) => {
                    // Send 200 Connection Established
                    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

                    // Start proxying data bidirectionally
                    let (mut client_read, mut client_write) = client.into_split();
                    let (mut upstream_read, mut upstream_write) = upstream_stream.into_split();

                    let client_to_upstream = tokio::io::copy(&mut client_read, &mut upstream_write);
                    let upstream_to_client = tokio::io::copy(&mut upstream_read, &mut client_write);

                    tokio::try_join!(client_to_upstream, upstream_to_client)?;
                }
                Err(e) => {
                    // Send 502 Bad Gateway
                    client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
                    return Err(e.into());
                }
            }
        }
    } else {
        // Send 405 Method Not Allowed
        client.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n").await?;
    }

    Ok(())
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
            if let Err(err) = handle_client(stream, upstream).await {
                eprintln!("Error handling client {}: {}", addr, err);
            }
        });
    }
}
