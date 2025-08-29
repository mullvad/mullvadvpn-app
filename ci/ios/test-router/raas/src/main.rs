use std::{fs, io, net::SocketAddr, path::Path, time::Duration};

mod block_list;
mod capture;
mod web;

#[tokio::main]
async fn main() {
    init_logging();
    create_temp_dir();

    let mut args = std::env::args().skip(1);
    let bind_address = args.next().expect("First arg must be listening address");

    let router =
        web::router(Default::default()).into_make_service_with_connect_info::<SocketAddr>();
    let listener = tokio::net::TcpListener::bind(bind_address)
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
            tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;

            if let Err(err) = capture::delete_old_captures().await {
                log::error!("Failed to delete old captures: {err}");
            }
        }
    });

    axum::serve(listener, router).await.unwrap();
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
