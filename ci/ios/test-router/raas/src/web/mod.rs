use std::sync::{Arc, Mutex};

use axum::{
    routing::{delete, post},
    Router,
};

use crate::block_list::BlockList;

pub mod routes;

pub fn router(block_list: BlockList) -> Router {
    Router::new()
        .route("/rule", post(routes::add_rule))
        .route("/remove-rules/:label", delete(routes::delete_rules))
        .with_state(Firewall { block_list: Arc::new(Mutex::new(block_list)) })
}

#[derive(Clone)]
pub(crate) struct Firewall {
    block_list: Arc<Mutex<BlockList>>,
}
