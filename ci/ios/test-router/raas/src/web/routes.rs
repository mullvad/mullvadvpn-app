use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ipnetwork::IpNetwork;
use mnl::mnl_sys::libc;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::block_list::{BlockList, BlockRule, Endpoints};
use crate::web;

#[derive(serde::Deserialize, Clone)]
pub struct NewRule {
    pub src: IpNetwork,
    pub dst: IpNetwork,
    pub protocols: Option<BTreeSet<TransportProtocol>>,
    #[serde(default)]
    pub block_wireguard: bool,
    pub label: Uuid,
}

#[derive(
    PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, serde::Deserialize, serde::Serialize,
)]
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
    Json(json): Json<NewRule>,
) -> impl IntoResponse {
    let result = access_firewall(state, move |fw| {
        let label = json.label;
        let src = json.src;
        let dst = json.dst;

        let rule = if json.block_wireguard {
            BlockRule::WireGuard {
                endpoints: Endpoints { src, dst },
            }
        } else {
            BlockRule::Host {
                endpoints: Endpoints { src, dst },
                protocols: json.protocols.unwrap_or_default(),
            }
        };

        fw.add_rule(rule.clone(), label)?;
        log_rule(&rule, &label);
        Ok(())
    })
    .await;

    respond_with_result(result, StatusCode::CREATED)
}

pub async fn delete_rules(
    Path(label): Path<Uuid>,
    State(state): State<super::State>,
) -> impl IntoResponse {
    let result = access_firewall(state, move |fw| {
        fw.clear_rules_with_label(&label)?;
        log::info!("Successfully removed all rules for test {label}");
        Ok(())
    })
    .await;

    respond_with_result(result, StatusCode::OK)
}

pub async fn access_firewall<F>(state: web::State, run: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut BlockList) -> anyhow::Result<()> + Send + 'static,
{
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let Ok(mut fw) = state.block_list.lock() else {
            return Err(anyhow::anyhow!("Firewall thread panicked"));
        };
        run(&mut fw)
    })
    .await
    .expect("failed to join blocking task")
}

fn respond_with_result(result: anyhow::Result<()>, success_code: StatusCode) -> impl IntoResponse {
    match result {
        Ok(_) => (success_code, String::new()),
        Err(err) => (StatusCode::SERVICE_UNAVAILABLE, format!("{err}\n")),
    }
}

pub async fn list_all_rules(State(state): State<super::State>) -> impl IntoResponse {
    let all_rules = tokio::task::spawn_blocking(move || -> anyhow::Result<BTreeMap<_, _>> {
        let Ok(fw) = state.block_list.lock() else {
            return Err(anyhow::anyhow!("Firewall thread panicked"));
        };

        Ok(fw.rules().clone())
    })
    .await
    .expect("failed to join blocking task");

    match all_rules {
        Ok(all_rules) => Json(all_rules).into_response(),
        Err(err) => (StatusCode::SERVICE_UNAVAILABLE, format!("{err}\n")).into_response(),
    }
}

fn log_rule(rule: &BlockRule, label: &Uuid) {
    match rule {
        BlockRule::Host {
            protocols,
            endpoints: Endpoints { src, dst },
        } => {
            log::info!(
                "Successfully added a rule to block {src} from {dst} for test {label} for protocols {protocols:?}",
            );
        }
        BlockRule::WireGuard {
            endpoints: Endpoints { src, dst },
        } => {
            log::info!("Successfully added a rule to block {src} from {dst} WireGuard traffic for test {label}",);
        }
    }
}
