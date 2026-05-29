use std::{fs, io, net::SocketAddr, path::Path, time::Duration};

mod capture;
mod firewall;
mod web;

#[tokio::main]
async fn main() {
    init_logging();
    create_temp_dir();

    let args = parse_args();

    #[cfg(target_os = "macos")]
    if let Some(host_ip) = args.dnat_host_ip {
        firewall::apply_dnat(host_ip).expect("Failed to apply DNAT rules");
    }

    let interface = args.interface.unwrap_or_else(|| {
        #[cfg(target_os = "linux")]
        {
            "any".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            panic!("--interface is required on macOS");
        }
    });

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

    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_hours(24)).await;

            if let Err(err) = capture::delete_old_captures().await {
                log::error!("Failed to delete old captures: {err}");
            }
        }
    });

    axum::serve(listener, router).await.unwrap();

    #[cfg(target_os = "macos")]
    firewall::cleanup_dnat();
}

struct Args {
    bind_address: String,
    interface: Option<String>,
    #[cfg(target_os = "macos")]
    dnat_host_ip: Option<std::net::Ipv4Addr>,
}

fn parse_args() -> Args {
    let mut args_iter = std::env::args().skip(1);
    let bind_address = args_iter
        .next()
        .expect("First arg must be listening address");

    let mut interface = None;
    #[cfg(target_os = "macos")]
    let mut dnat_host_ip = None;

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--interface" => {
                interface = Some(
                    args_iter
                        .next()
                        .expect("--interface requires an argument"),
                );
            }
            #[cfg(target_os = "macos")]
            "--host-ip" => {
                dnat_host_ip = Some(
                    args_iter
                        .next()
                        .expect("--host-ip requires an argument")
                        .parse()
                        .expect("--host-ip must be a valid IPv4 address"),
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
        dnat_host_ip,
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
