use std::sync::{Arc, Mutex};

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{capture::Capture, firewall::BlockList};

mod capture;
mod ip;
pub mod routes;

pub fn router(block_list: BlockList, interface: String) -> Router {
    Router::new()
        .route("/own-ip", get(ip::host_ip))
        .route("/rules", get(routes::list_all_rules))
        .route("/rule", post(routes::add_rule))
        .route("/remove-rules/:label", delete(routes::delete_rules))
        .route("/capture", post(capture::start))
        .route("/stop-capture/:label", post(capture::stop))
        .route("/last-capture/:label", get(capture::get))
        .route("/parse-capture/:label", put(capture::parse))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(State {
            block_list: Arc::new(Mutex::new(block_list)),
            capture: Arc::new(tokio::sync::Mutex::new(Capture::new(interface))),
        })
}

#[derive(Clone)]
pub struct State {
    block_list: Arc<Mutex<BlockList>>,
    capture: Arc<tokio::sync::Mutex<Capture>>,
}
