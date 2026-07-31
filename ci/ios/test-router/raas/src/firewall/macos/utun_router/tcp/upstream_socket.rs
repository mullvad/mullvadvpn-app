use bytes::{Bytes, BytesMut};
use smoltcp::wire::{IpProtocol, Ipv4Packet, TcpPacket};
use std::{collections::VecDeque, mem, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    task::JoinHandle,
    time::timeout,
};

use super::TcpConnectionId;

pub struct UpstreamSocketHandle {
    pub recv_task: JoinHandle<()>,
    // channel for sending data to the upstream TCP socket immediately.
    pub downstream_payload_tx: mpsc::Sender<Bytes>,
    // IP packets to be written into smoltcp
    pub downstream_buffer: VecDeque<Bytes>,
    // TCP payloads to be written into smoltcp socket
    pub upstream_buffer: VecDeque<Bytes>,
    pub is_connected: bool,
}

impl UpstreamSocketHandle {
    pub fn new(connection_id: TcpConnectionId, socket_tx: mpsc::Sender<UpstreamSocketMsg>) -> Self {
        let (send_tx, mut recv_rx) = mpsc::channel(1024);
        let recv_task = tokio::spawn(async move {
            let mut connection = match timeout(
                Duration::from_secs(60),
                TcpStream::connect(connection_id.upstream),
            )
            .await
            {
                Ok(Ok(connection)) => connection,
                Ok(Err(err)) => {
                    log::error!("{connection_id:?} connection failed {err}");
                    let _ = socket_tx
                        .send(UpstreamSocketMsg {
                            id: connection_id,
                            cmd: UpstreamSocketCommand::Failed,
                        })
                        .await;
                    return;
                }
                Err(err) => {
                    log::error!("Connection {connection_id:?} timed out");
                    let _ = socket_tx
                        .send(UpstreamSocketMsg {
                            id: connection_id,
                            cmd: UpstreamSocketCommand::Failed,
                        })
                        .await;
                    return;
                }
            };

            let _ = socket_tx.send(UpstreamSocketMsg{
                id: connection_id,
                cmd: UpstreamSocketCommand::Connected,
            }).await;

            let mut read_buf = BytesMut::with_capacity(u16::MAX.into());
            let (mut reader, mut writer) = connection.split();
            loop {
                tokio::select! {
                    // to decouple sending and receiving, this select arm should be moved into it's
                    // own tokio task. Thus, we can transform the existing one into a plain
                    // `while let Some(payload) = reader.recv_buf(&mut read_buf)`
                    new_packet = recv_rx.recv() => {
                        let packet = match new_packet {
                            Some(payload) => payload,
                            None => {
                                log::debug!("{connection_id:?} writer is closed, shutting down");
                                let _ = writer.shutdown().await;
                                return;
                            }
                        };

                        let Some(payload) = extract_tcp_payload(&packet) else {
                            log::error!("{connection_id:?} - Received a packet with no payload");
                            continue;
                        };

                        if let Err(err) = writer.write(payload).await {
                            log::error!("{connection_id:?} - Failed to send payload to upstream: {err}");
                            let _ = socket_tx.send(UpstreamSocketMsg {
                                id: connection_id,
                                cmd: UpstreamSocketCommand::Failed,
                            }).await;
                        };

                        let _ = socket_tx.send(UpstreamSocketMsg {
                            id: connection_id,
                            cmd: UpstreamSocketCommand::SentPayload(packet),
                        }).await;
                    },

                    read_result = reader.read_buf(&mut read_buf) => {
                        match read_result {
                            Ok(_) => {
                                let received_payload = read_buf.split().freeze();
                                let _ = socket_tx.send(UpstreamSocketMsg {
                                    id: connection_id,
                                    cmd: UpstreamSocketCommand::ReceivedPayload(received_payload)
                                }).await;
                            },
                            Err(err) => {
                                log::error!("{connection_id:?} Failed to receive data from upstream socket: {err}");
                                let _ = socket_tx.send(UpstreamSocketMsg {
                                    id: connection_id,
                                    cmd: UpstreamSocketCommand::Failed
                                }).await;

                            }
                        };
                    },
                }
            }
        });

        Self {
            downstream_payload_tx: send_tx,
            recv_task,
            upstream_buffer: VecDeque::new(),
            downstream_buffer: VecDeque::new(),
            is_connected: false,
        }
    }
}

fn extract_tcp_payload(packet: &Bytes) -> Option<&[u8]> {
    let ipv4_packet = Ipv4Packet::new_checked(packet).ok()?;
    let IpProtocol::Tcp = ipv4_packet.next_header() else {
        log::error!("Received a non-tcp packet");
        return None;
    };

    let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload()).ok()?;
    let payload = tcp_packet.payload();
    if payload.is_empty() {
        log::error!("Received TCP packet with empty payload");
        return None;
    }

    Some(payload)
}

pub struct UpstreamSocketMsg {
    pub id: TcpConnectionId,
    pub cmd: UpstreamSocketCommand,
}

pub enum UpstreamSocketCommand {
    Connected,
    Failed,
    // Received payload from upstream
    ReceivedPayload(Bytes),
    // Successfully sent payload to upstream
    SentPayload(Ipv4Packet<Bytes>),
}
