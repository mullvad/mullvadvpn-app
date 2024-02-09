mod block_list;
mod web;

#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::from_env(env_logger::DEFAULT_FILTER_ENV);
    builder
        .filter(None, log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .format_timestamp(None)
        .init();

    let mut args = std::env::args().skip(1);
    let bind_address = args.next().expect("First arg must be listening address");

    let router = web::router(Default::default());
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .expect("Failed to bind to listening socket");
    log::info!(
        "listening on {}",
        listener
            .local_addr()
            .expect("Failed to get local address of TCP socket")
    );

    axum::serve(listener, router).await.unwrap();
}
