use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ipnetwork::IpNetwork;
use mnl::mnl_sys::libc;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::{
    block_list::{BlockList, BlockRule, Endpoints},
    web,
};

#[derive(serde::Deserialize, Clone)]
pub struct NewRule {
    /// A packet that is sent *from* `src` will match the block rule.
    pub src: IpNetwork,
    /// A packet that is sent *to* `dst` will match the block rule.
    pub dst: IpNetwork,
    /// A list of protocols that should be blocked, e.g. Tcp or WireGuard. The default behavior
    /// is to block all traffic regardless of protocol, but if `protocols` is non-empty, only
    /// traffic that uses that protocol is blocked.
    #[serde(default)]
    pub protocols: BTreeSet<Protocol>,
    /// A unique identifier for a group of rules. It is possible to add rules to an existing label
    /// and to remove all rules for a label.
    pub label: Uuid,
    /// Normally a packet sent to `dst` would match the block rule, but this option inverts that
    /// so that any packet *not* sent to `dst` will match the block rule.
    #[serde(default)]
    pub block_all_except_dst: bool,
}

#[derive(
    PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum Protocol {
    Transport(TransportProtocol),
    Application(AppProtocol),
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

#[derive(
    PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AppProtocol {
    #[serde(rename = "wireguard")]
    WireGuard,
}

impl AppProtocol {
    // Each "app protocol" (e.g. WireGuard, more could be added in the future) is mapped to its
    // own block rule, as opposed to the transport protocols which have a single rule.
    // This is to support each app protocol being able to have a unique block criteria
    // (e.g. WireGuard needs to block UDP packets that match a certain pattern).
    fn as_block_rule(&self, endpoints: Endpoints) -> BlockRule {
        match self {
            AppProtocol::WireGuard => BlockRule::WireGuard { endpoints },
        }
    }
}

pub async fn add_rule(
    State(state): State<super::State>,
    Json(json): Json<NewRule>,
) -> impl IntoResponse {
    let result = access_firewall(state, move |fw| {
        let label = json.label;
        let endpoints = Endpoints {
            src: json.src,
            dst: json.dst,
            invert_dst: json.block_all_except_dst,
        };
        let protocols = json.protocols;

        let mut block_rules = vec![];

        if protocols.is_empty() {
            // If no protocols are specified we default to blocking everything for (src, dst).
            block_rules.push(BlockRule::Host {
                endpoints,
                protocols: BTreeSet::new(),
            });
        } else {
            let mut transport = BTreeSet::new();
            let mut application = BTreeSet::new();
            for protocol in protocols {
                match protocol {
                    Protocol::Transport(p) => transport.insert(p),
                    Protocol::Application(p) => application.insert(p),
                };
            }
            if !transport.is_empty() {
                block_rules.push(BlockRule::Host {
                    endpoints,
                    protocols: transport,
                });
            }
            for protocol in application {
                block_rules.push(protocol.as_block_rule(endpoints));
            }
        }

        fw.add_rules(&block_rules, label)?;
        for rule in block_rules {
            log_rule(&rule, &label);
        }
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
            endpoints:
                Endpoints {
                    src,
                    dst,
                    invert_dst,
                },
        } => {
            log::info!(
                "Successfully added a rule to {} {src} to {dst} for protocols {protocols:?} [test: {label}]",
                if *invert_dst { "allow only traffic from" } else { "block" },
            );
        }
        BlockRule::WireGuard {
            endpoints:
                Endpoints {
                    src,
                    dst,
                    invert_dst,
                },
        } => {
            log::info!(
                "Successfully added a rule to {} {src} to {dst} for WireGuard [test: {label}]",
                if *invert_dst {
                    "allow only traffic from"
                } else {
                    "block"
                },
            );
        }
    }
}
