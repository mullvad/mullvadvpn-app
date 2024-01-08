use std::{io, time::Duration};

use futures::{channel::mpsc, future::BoxFuture, pin_mut, FutureExt, SinkExt, StreamExt};
use mullvad_management_interface::{ManagementServiceClient, MullvadProxyClient};
use test_rpc::{
    mullvad_daemon::MullvadClientVersion,
    transport::{ConnectionHandle, GrpcForwarder},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use tower::Service;

const GRPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CONVERTER_BUF_SIZE: usize = 16 * 1024;

#[derive(Clone)]
struct DummyService {
    management_channel_provider_tx: mpsc::UnboundedSender<DuplexStream>,
}

impl<Request> Service<Request> for DummyService {
    type Response = DuplexStream;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<DuplexStream, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Request) -> Self::Future {
        log::trace!("DummyService::call");

        let (channel_in, channel_out) = tokio::io::duplex(CONVERTER_BUF_SIZE);
        let notifier_tx = self.management_channel_provider_tx.clone();

        Box::pin(async move {
            notifier_tx
                .unbounded_send(channel_in)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "stream receiver is down"))?;
            Ok(channel_out)
        })
    }
}

#[derive(Clone)]
pub struct RpcClientProvider {
    service: DummyService,
}

impl RpcClientProvider {
    pub async fn as_type(
        &self,
        client_type: MullvadClientVersion,
    ) -> Box<dyn std::any::Any + Send> {
        match client_type {
            MullvadClientVersion::New => Box::new(self.new_client().await),
            MullvadClientVersion::None => Box::new(()),
        }
    }

    pub async fn new_client(&self) -> MullvadProxyClient {
        // FIXME: Ugly workaround to ensure that we don't receive stuff from a
        // previous RPC session.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        log::debug!("Mullvad daemon: connecting");
        let channel = tonic::transport::Endpoint::from_static("serial://placeholder")
            .timeout(GRPC_REQUEST_TIMEOUT)
            .connect_with_connector(self.service.clone())
            .await
            .unwrap();

        MullvadProxyClient::from_rpc_client(ManagementServiceClient::new(channel))
    }
}

pub fn new_rpc_client(
    connection_handle: ConnectionHandle,
    mullvad_daemon_transport: GrpcForwarder,
) -> RpcClientProvider {
    let mut framed_transport = LengthDelimitedCodec::new().framed(mullvad_daemon_transport);
    let (management_channel_provider_tx, mut management_channel_provider_rx) = mpsc::unbounded();

    tokio::spawn(async move {
        let mut read_buf = [0u8; CONVERTER_BUF_SIZE];
        loop {
            log::trace!("waiting for management interface client");

            let mut management_channel_in: DuplexStream =
                match management_channel_provider_rx.next().await {
                    Some(channel) => channel,
                    None => {
                        log::trace!("exiting management interface forward loop");
                        break;
                    }
                };

            // clear data from last session
            while let Some(_next) = framed_transport.next().now_or_never() {}

            loop {
                let proxy_read = management_channel_in.read(&mut read_buf);
                pin_mut!(proxy_read);

                let reset_notified = connection_handle.notified_reset();
                pin_mut!(reset_notified);

                match futures::future::select(
                    reset_notified,
                    futures::future::select(framed_transport.next(), proxy_read),
                )
                .await
                {
                    futures::future::Either::Left(_) => {
                        log::debug!("Restarting daemon RPC client");
                        break;
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Left((Some(Ok(bytes)), _)),
                        _,
                    )) => {
                        if bytes.is_empty() {
                            log::trace!("Management channel EOF");

                            if let Err(error) = management_channel_in.shutdown().await {
                                log::error!("Failed to shut down forwarder stream: {}", error);
                            }
                            break;
                        }
                        if management_channel_in.write_all(&bytes).await.is_err() {
                            break;
                        }
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Left((Some(Err(error)), _)),
                        _,
                    )) => {
                        log::debug!("Management channel stream errored: {}", error);
                        break;
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Left((None, _)),
                        _,
                    )) => break,
                    futures::future::Either::Right((
                        futures::future::Either::Right((Ok(num_bytes), _)),
                        _,
                    )) => {
                        if framed_transport
                            .send(read_buf[..num_bytes].to_vec().into())
                            .await
                            .is_err()
                        {
                            break;
                        }
                        if num_bytes == 0 {
                            log::trace!("Mullvad daemon connection EOF");
                            break;
                        }
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Right((Err(_), _)),
                        _,
                    )) => {
                        let _ = framed_transport.send(bytes::Bytes::new()).await;
                        break;
                    }
                }
            }
        }
    });

    let service = DummyService {
        management_channel_provider_tx,
    };

    RpcClientProvider { service }
}
