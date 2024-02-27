use std::sync::{Arc, Mutex};

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::block_list::BlockList;
use crate::capture::Capture;

mod capture;
pub mod routes;

pub fn router(block_list: BlockList) -> Router {
    Router::new()
        .route("/rule", post(routes::add_rule))
        .route("/remove-rules/:label", delete(routes::delete_rules))
        .route("/capture", post(capture::start))
        .route("/stop-capture/:label", post(capture::stop))
        .route("/last-capture/:label", get(capture::get))
        .with_state(State {
            block_list: Arc::new(Mutex::new(block_list)),
            capture: Arc::new(tokio::sync::Mutex::new(Capture::default())),
        })
}

#[derive(Clone)]
pub(crate) struct State {
    block_list: Arc<Mutex<BlockList>>,
    capture: Arc<tokio::sync::Mutex<Capture>>,
}
