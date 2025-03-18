use std::sync::{Arc, Mutex};

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{block_list::BlockList, capture::Capture};

mod firewall;
mod ip;
pub mod routes;

pub fn router(block_list: BlockList) -> Router {
    Router::new()
        .route("/own-ip", get(ip::host_ip))
        .route("/rules", get(routes::list_all_rules))
        .route("/rule", post(routes::add_rule))
        .route("/block-wireguard", post(routes::block_wireguard_rule))
        .route("/remove-rules/:label", delete(routes::delete_rules))
        .route("/capture", post(firewall::start))
        .route("/stop-capture/:label", post(firewall::stop))
        .route("/last-capture/:label", get(firewall::get))
        .route("/parse-capture/:label", put(firewall::parse))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(State {
            block_list: Arc::new(Mutex::new(block_list)),
            capture: Arc::new(tokio::sync::Mutex::new(Capture::default())),
        })
}

#[derive(Clone)]
pub struct State {
    block_list: Arc<Mutex<BlockList>>,
    capture: Arc<tokio::sync::Mutex<Capture>>,
}
