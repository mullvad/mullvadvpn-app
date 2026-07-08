use std::{fs, io, net::SocketAddr, path::Path, time::Duration};

use smoltcp::wire::Ipv4Packet;

mod capture;
mod firewall;
mod web;

#[tokio::main]
async fn main() {
    init_logging();
    create_temp_dir();

    let args = parse_args();

    #[cfg(target_os = "macos")]
    let tun_device =
        firewall::setup_utun(args.client_ip).expect("Failed to create a tunnel device");

    let interface = {
        #[cfg(target_os = "linux")]
        {
            args.interface.clone()
        }
        #[cfg(target_os = "macos")]
        {
            tun_device.name().expect("Failed to read device name")
        }
    };

    let router = web::router(Default::default(), interface)
        .into_make_service_with_connect_info::<SocketAddr>();
    let listener = tokio::net::TcpListener::bind(&args.bind_address)
        .await
        .expect("Failed to bind to listening socket");
    log::info!(
        "listening on {}",
        listener
            .local_addr()
            .expect("Failed to get local address of TCP socket")
    );

    #[cfg(target_os = "macos")]
    tokio::spawn(async move {
        let mut read_buf = vec![0u8; 2000];
        while let Ok(bytes_received) = tun_device.recv(&mut read_buf).await {
            let packet = Ipv4Packet::new_unchecked(&read_buf[..bytes_received]);
            let src = packet.src_addr();
            let dst = packet.dst_addr();
            println!("Received packet from utun - with src {src} and dst {dst}");
        }
    });

    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_hours(24)).await;

            if let Err(err) = capture::delete_old_captures().await {
                log::error!("Failed to delete old captures: {err}");
            }
        }
    });

    axum::serve(listener, router).await.unwrap();
}

struct Args {
    bind_address: String,
    interface: Option<String>,
    #[cfg(target_os = "macos")]
    client_ip: std::net::Ipv4Addr,
}

fn parse_args() -> Args {
    // TODO: use clap for parsing args instead
    let mut args_iter = std::env::args().skip(1);
    let bind_address = args_iter
        .next()
        .expect("First arg must be listening address");

    let mut interface = None;
    #[cfg(target_os = "macos")]
    let mut client_ip = None;

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--interface" => {
                interface = Some(args_iter.next().expect("--interface requires an argument"));
            }
            #[cfg(target_os = "macos")]
            "--client-ip" => {
                client_ip = Some(
                    args_iter
                        .next()
                        .expect("--client-ip requires an argument")
                        .parse()
                        .expect("--client-ip must be a valid IPv4 address"),
                );
            }
            other => {
                panic!("Unknown argument: {other}");
            }
        }
    }

    Args {
        bind_address,
        interface,
        #[cfg(target_os = "macos")]
        client_ip: client_ip.unwrap(),
    }
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_env(env_logger::DEFAULT_FILTER_ENV);
    builder
        .filter(None, log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .format_timestamp(None)
        .init();
}

fn create_temp_dir() {
    let tmp_dir = std::env::temp_dir().join("raas");
    create_dir_if_not_exist(tmp_dir).expect("Failed to create tmp directory");
}

fn create_dir_if_not_exist<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();

    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        create_dir_if_not_exist(parent)?;
    }

    fs::create_dir(path)?;
    Ok(())
}
