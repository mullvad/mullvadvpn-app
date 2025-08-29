use axum::{extract::ConnectInfo, response::IntoResponse};
use std::net::SocketAddr;

/// Returns IP address of caller as a string
pub async fn host_ip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    addr.ip().to_string()
}
