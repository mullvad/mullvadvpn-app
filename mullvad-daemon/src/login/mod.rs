//! Remote login - P2P using iroh.
use std::pin::Pin;
use std::time::Duration;

use anyhow::Context;
use iroh::{Endpoint, SecretKey, endpoint::presets};
use mullvad_types::account::AccountNumber;
use tokio::time::timeout;

pub use iroh_tickets::endpoint::EndpointTicket;

/// The ALPN for mullvad.
///
/// It is basically just passing data through 1:1, except that the connecting
/// side will send a fixed size handshake to make sure the stream is created.
const ALPN: &[u8] = b"MULLVAD";

const ONLINE_TIMEOUT: Duration = Duration::from_secs(5);

/// Connect to a peer and send them our account number.
pub async fn login(ticket: EndpointTicket, account: AccountNumber) -> anyhow::Result<()> {
    let secret_key = create_secret();
    let endpoint = create_endpoint(secret_key, vec![]).await?;
    let addr = ticket.endpoint_addr();
    let remote_endpoint_id = addr.id;
    // connect to the remote, try only once
    let connection = endpoint.connect(addr.clone(), ALPN).await?;
    log::trace!("connected to {}", remote_endpoint_id);
    // open a uni-directional stream, try only once
    let mut s = connection.open_uni().await?;
    log::trace!("opened uni stream to {}", remote_endpoint_id);
    // TODO: Handshake?
    // the connecting side must write first, so proceed by sending the account number.
    log::trace!("Sending account number ..");
    s.write_all(account.as_bytes()).await?;
    log::trace!("Account number sent");
    // A well-behaved receiving end should close their end of the channel once they have read the
    // account number. Wait for receiving end to close their channel to know that they have ack'd
    // our message.
    if let Ok(Some(error)) = s.stopped().await {
        log::error!("Peer stopped the stream: {error}");
    }
    // Signal to receiver that we won't be sending any more data.
    s.finish()?;
    Ok(())
}

/// Start listening for an incoming peer connection. When a peer connects, try to receive a login
/// token from them.
pub async fn init_login() -> anyhow::Result<(
    EndpointTicket,
    Pin<Box<dyn Future<Output = anyhow::Result<AccountNumber>> + Send>>,
)> {
    let secret_key = create_secret();
    let endpoint = create_endpoint(secret_key, vec![ALPN.to_vec()]).await?;
    // wait for the endpoint to figure out its home relay and addresses before making a ticket
    timeout(ONLINE_TIMEOUT, endpoint.online())
        .await
        .context(anyhow::anyhow!("Failed to connect to the home relay"))?;
    let addr = endpoint.addr();
    let ticket = EndpointTicket::new(addr);

    // Wait for a handshake immediately followed by an account number.
    let login_fut = Box::pin(async move {
        loop {
            let Some(connecting) = endpoint.accept().await else {
                break;
            };
            let connection = match connecting.await {
                Ok(connection) => connection,
                Err(cause) => {
                    log::warn!("error accepting connection: {}", cause);
                    // if accept fails, we want to continue accepting connections
                    continue;
                }
            };
            let remote_endpoint_id = &connection.remote_id();
            log::trace!("got connection from {}", remote_endpoint_id);
            let mut r = match connection.accept_uni().await {
                Ok(receive_stream) => receive_stream,
                Err(cause) => {
                    // TODO: if accept_uni fails, we want to quit.
                    log::error!("error accepting stream: {cause}");
                    continue;
                }
            };
            log::trace!("accepted unidirectional stream from {}", remote_endpoint_id);
            // TODO: read the handshake and verify it
            log::trace!("Listening for account number");
            let account_number = {
                let mut account_number_buf = [0u8; 16];
                r.read_exact(&mut account_number_buf).await?;
                log::trace!("Read <{account_number_buf:#?}> from stream");
                AccountNumber::from_utf8(account_number_buf.to_vec())?
            };
            log::debug!("Received account number {account_number}");
            // stop accepting connections after the first successful one.
            // Signal that we will not be receiving any more bytes.
            // TODO: Magic number
            if let Err(close_error) = timeout(Duration::from_secs(5), endpoint.close()).await {
                log::warn!("Failed to gracefully close QUIC connection: {close_error}");
            }
            return Ok(account_number);
        }
        anyhow::bail!("Failed to read account number from remote peer");
    });

    Ok((ticket, login_fut))
}

/// Create a new iroh endpoint.
async fn create_endpoint(secret_key: SecretKey, alpns: Vec<Vec<u8>>) -> anyhow::Result<Endpoint> {
    let builder = Endpoint::builder(presets::N0)
        .secret_key(secret_key)
        .alpns(alpns);
    let endpoint = builder.bind().await?;
    Ok(endpoint)
}

/// Generate a secret key.
fn create_secret() -> SecretKey {
    SecretKey::generate(&mut rand::rng())
}
