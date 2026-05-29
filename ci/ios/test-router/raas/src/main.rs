#[cfg(target_os = "linux")]
use std::convert::Infallible;
use std::{fmt::Display, fs, io, net::SocketAddr, path::Path, time::Duration};

mod capture;
mod firewall;
mod web;

use firewall::BlockList;
use tokio::signal;

#[tokio::main]
async fn main() {
    init_logging();
    create_temp_dir();

    let args = parse_args();

    #[cfg(target_os = "macos")]
    let (tunnel_device, firewall) =
        firewall::setup_utun().expect("Failed to create a tunnel device");

    let interface = {
        #[cfg(target_os = "linux")]
        {
            args.interface
                .clone()
                .expect("No interface argument supplied")
        }
        #[cfg(target_os = "macos")]
        {
            tun::AbstractDevice::tun_name(&*tunnel_device.source).unwrap()
        }
    };

    let router = web::router(
        create_block_list(
            #[cfg(target_os = "macos")]
            tunnel_device,
        ),
        interface,
    )
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

    #[cfg(target_os = "macos")]
    let platform_cleanup = move || -> Result<(), io::Error> { firewall.reset() };

    #[cfg(target_os = "linux")]
    let platform_cleanup = || -> Result<(), Infallible> { Ok(()) };

    let shutdown = graceful_shutdown(platform_cleanup);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}

async fn graceful_shutdown<E: Display>(cleanup: impl FnOnce() -> Result<(), E>) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    println!("Ctrl+C received, shutting down...");

    if let Err(e) = cleanup() {
        eprintln!("cleanup failed: {e}");
    }
}

struct Args {
    bind_address: String,
    // The capture interface is only configurable on Linux; macOS derives everything itself.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    interface: Option<String>,
}

fn parse_args() -> Args {
    // TODO: use clap for parsing args instead
    let mut args_iter = std::env::args().skip(1);
    let bind_address = args_iter
        .next()
        .expect("First arg must be listening address");

    let mut interface = None;

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--interface" => {
                interface = Some(args_iter.next().expect("--interface requires an argument"));
            }
            other => {
                panic!("Unknown argument: {other}");
            }
        }
    }

    Args {
        bind_address,
        interface,
    }
}

fn init_logging() {
    // Defaulting through the filter rather than overriding it keeps `RUST_LOG` working, which is
    // what turns on the per-packet logs.
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    builder
        .write_style(env_logger::WriteStyle::Always)
        .format_timestamp_millis()
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

#[cfg(target_os = "linux")]
fn create_block_list() -> BlockList {
    Default::default()
}

#[cfg(target_os = "macos")]
fn create_block_list(tunnel_device: crate::firewall::macos::TunnelDevices) -> BlockList {
    BlockList::new(tunnel_device)
}
