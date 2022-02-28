use crate::DaemonEventSender;
use futures::{channel::oneshot, stream, Stream, StreamExt};
use mullvad_rpc::proxy::ApiConnectionMode;
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;

pub(crate) struct ApiConnectionModeRequest {
    pub response_tx: oneshot::Sender<ApiConnectionMode>,
    pub retry_attempt: u32,
}

/// Returns a stream that returns the next API bridge to try.
/// `initial_config` refers to the first config returned by the stream. The daemon is not notified
/// of this.
pub(crate) fn create_api_config_provider(
    daemon_sender: DaemonEventSender<ApiConnectionModeRequest>,
    initial_config: ApiConnectionMode,
) -> impl Stream<Item = ApiConnectionMode> + Unpin {
    struct Context {
        attempt: u32,
        daemon_sender: DaemonEventSender<ApiConnectionModeRequest>,
    }

    let ctx = Context {
        attempt: 1,
        daemon_sender,
    };

    Box::pin(
        stream::once(async move { initial_config }).chain(stream::unfold(
            ctx,
            |mut ctx| async move {
                ctx.attempt = ctx.attempt.wrapping_add(1);
                let (response_tx, response_rx) = oneshot::channel();

                let _ = ctx.daemon_sender.send(ApiConnectionModeRequest {
                    response_tx,
                    retry_attempt: ctx.attempt,
                });

                let new_config = response_rx.await.unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to receive API proxy config")
                    );
                    // Fall back on unbridged connection
                    ApiConnectionMode::Direct
                });

                Some((new_config, ctx))
            },
        )),
    )
}
