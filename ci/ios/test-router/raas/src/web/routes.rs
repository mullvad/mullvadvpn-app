use std::{collections::BTreeSet, net::IpAddr};

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use mnl::mnl_sys::libc;
use uuid::Uuid;

use crate::block_list::BlockRule;

#[derive(serde::Deserialize, Clone)]
pub struct NewRule {
    pub from: IpAddr,
    pub to: IpAddr,
    pub protocols: Option<BTreeSet<TransportProtocol>>,
    pub label: Uuid,
}

#[derive(serde::Deserialize, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    IcmpV6,
}

impl TransportProtocol {
    pub fn as_ipproto(&self) -> u8 {
        match self {
            TransportProtocol::Udp => libc::IPPROTO_UDP as u8,
            TransportProtocol::Tcp => libc::IPPROTO_TCP as u8,
            TransportProtocol::Icmp => libc::IPPROTO_ICMP as u8,
            TransportProtocol::IcmpV6 => libc::IPPROTO_ICMPV6 as u8,
        }
    }
}

pub async fn add_rule(
    State(state): State<super::State>,
    Json(rule): Json<NewRule>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let label = rule.label;
        let rule = BlockRule {
            src: rule.from,
            dst: rule.to,
            protocols: rule.protocols.unwrap_or_default(),
        };
        let Ok(mut fw) = state.block_list.lock() else {
            return Err(anyhow::anyhow!("Firewall thread panicked"));
        };

        fw.add_rule(rule.clone(), label)?;
        log::info!(
            "Successfully added a rule to block {} from {} for test {}",
            rule.src,
            rule.dst,
            label,
        );
        Ok(())
    })
    .await
    .expect("failed to join blocking task");

    respond_with_result(result, StatusCode::CREATED)
}

pub async fn delete_rules(
    Path(label): Path<Uuid>,
    State(state): State<super::State>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let Ok(mut fw) = state.block_list.lock() else {
            return Err(anyhow::anyhow!("Firewall thread panicked"));
        };

        fw.clear_rules_with_label(&label)?;
        log::info!("Successfully removed all rules for test {label}",);
        Ok(())
    })
    .await
    .expect("failed to join blocking task");
    respond_with_result(result, StatusCode::OK)
}

fn respond_with_result(result: anyhow::Result<()>, success_code: StatusCode) -> impl IntoResponse {
    match result {
        Ok(_) => (success_code, String::new()),
        Err(err) => (StatusCode::SERVICE_UNAVAILABLE, format!("{err}\n")),
    }
}
