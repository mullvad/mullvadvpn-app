use crate::{
    rest,
    rest::{RequestFactory, RequestServiceHandle},
};
use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use hyper::StatusCode;
use mullvad_types::account::{AccessToken, AccessTokenData, AccountToken};
use std::{collections::HashMap, sync::Arc};
use tokio::select;

pub const AUTH_URL_PREFIX: &str = "auth/v1";

#[derive(Clone)]
pub struct AccessTokenStore {
    tx: mpsc::UnboundedSender<StoreAction>,
}

enum StoreAction {
    /// Request an access token for `AccountToken`, or return a saved one if it's not expired.
    GetAccessToken(
        AccountToken,
        oneshot::Sender<Result<AccessToken, Arc<rest::Error>>>,
    ),
    /// Forget cached access token for `AccountToken`, and drop any in-flight requests
    InvalidateToken(AccountToken),
}

impl AccessTokenStore {
    pub(crate) fn new(service: RequestServiceHandle, factory: RequestFactory) -> Self {
        let (tx, rx) = mpsc::unbounded();
        tokio::spawn(Self::service_requests(rx, service, factory));
        Self { tx }
    }

    async fn service_requests(
        mut rx: mpsc::UnboundedReceiver<StoreAction>,
        service: RequestServiceHandle,
        factory: RequestFactory,
    ) {
        let mut access_from_account: HashMap<String, AccessTokenData> = HashMap::new();
        let mut inflight_requests = HashMap::new();
        let mut response_channels = HashMap::new();

        let (completed_tx, mut completed_rx) = mpsc::unbounded();

        loop {
            select! {
                action = rx.next() => {
                    let Some(action) = action else {
                        // We're done
                        break;
                    };

                    match action {
                        StoreAction::GetAccessToken(account, response_tx) => {
                            // If there is an unexpired access token, just return it.
                            // Otherwise, generate a new token
                            if let Some(access_token) = access_from_account.get_mut(&account) {
                                if !access_token.is_expired() {
                                    log::trace!("Using stored access token");
                                    let _ = response_tx.send(Ok(access_token.access_token.clone()));
                                    continue;
                                }

                                log::debug!("Replacing expired access token");
                                access_from_account.remove(&account);
                            }

                            // Begin requesting an access token if it's not already underway.
                            // If there's already an inflight request, just save `response_tx`
                            inflight_requests
                                .entry(account.clone())
                                .or_insert_with(|| {
                                    let completed_tx = completed_tx.clone();
                                    let account = account.clone();
                                    let service = service.clone();
                                    let factory = factory.clone();

                                    log::debug!("Fetching access token for an account");

                                    tokio::spawn(async move {
                                        let result = fetch_access_token(service, factory, account.clone()).await;
                                        let _ = completed_tx.unbounded_send((account, result));
                                    })
                                });

                            // Save the channel to respond to later
                            response_channels
                                .entry(account)
                                .or_insert_with(Vec::new)
                                .push(response_tx);
                        }
                        StoreAction::InvalidateToken(account) => {
                            // Drop in-flight requests for the account
                            // & forget any existing access token

                            log::debug!("Invalidating access token for an account");

                            if let Some(task) = inflight_requests.remove(&account) {
                                task.abort();
                                let _ = task.await;
                            }

                            response_channels.remove(&account);
                            access_from_account.remove(&account);
                        }
                    }
                }

                Some((account, result)) = completed_rx.next() => {
                    inflight_requests.remove(&account);

                    // Sadly, rest::Error is not cloneable
                    let result = result.map_err(Arc::new);

                    // Send response to all channels
                    if let Some(channels) = response_channels.remove(&account) {
                        for tx in channels {
                            let _ = tx.send(result.clone().map(|data| data.access_token));
                        }
                    }

                    if let Ok(access_token) = result {
                        access_from_account.insert(account, access_token);
                    }
                }
            }
        }
    }

    /// Obtain access token for an account, requesting a new one from the API if necessary.
    pub async fn get_token(&self, account: &AccountToken) -> Result<AccessToken, Arc<rest::Error>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .unbounded_send(StoreAction::GetAccessToken(account.to_owned(), tx));
        rx.await.map_err(|_| rest::Error::Aborted)?
    }

    /// Remove an access token if the API response calls for it.
    pub fn check_response<T>(&self, account: &AccountToken, response: &Result<T, rest::Error>) {
        if let Err(rest::Error::ApiError(_status, code)) = response {
            if code == crate::INVALID_ACCESS_TOKEN {
                let _ = self
                    .tx
                    .unbounded_send(StoreAction::InvalidateToken(account.to_owned()));
            }
        }
    }
}

async fn fetch_access_token(
    service: RequestServiceHandle,
    factory: RequestFactory,
    account_token: AccountToken,
) -> Result<AccessTokenData, rest::Error> {
    #[derive(serde::Serialize)]
    struct AccessTokenRequest {
        account_number: String,
    }
    let request = AccessTokenRequest {
        account_number: account_token,
    };

    let rest_request = factory.post_json(&format!("{AUTH_URL_PREFIX}/token"), &request)?;
    let response = service.request(rest_request).await?;
    let response = rest::parse_rest_response(response, &[StatusCode::OK]).await?;
    rest::deserialize_body(response).await
}
