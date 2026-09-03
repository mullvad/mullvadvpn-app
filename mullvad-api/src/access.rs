use crate::rest::{self, handle_error_response, hyper_request_json_bytes};
use http::Method;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, client::conn::http1};
use mullvad_types::account::{AccessToken, AccessTokenData, AccountNumber};
use std::collections::{HashMap, hash_map::Entry};

pub const ACCESS_TOKEN_PATH: &str = "auth/v1/token";

#[derive(Default)]
pub(crate) struct AccessTokenStore {
    tokens: HashMap<AccountNumber, AccessTokenData>,
}

impl AccessTokenStore {
    /// Obtain access token for an account, requesting a new one from the API if necessary.
    pub(crate) async fn get_token(
        &mut self,
        account: &AccountNumber,
        host: &str,
        send_request: &mut http1::SendRequest<Full<Bytes>>,
    ) -> Result<AccessToken, rest::Error> {
        // If there is an unexpired access token, just return it.
        // Otherwise, generate a new token
        if let Entry::Occupied(entry) = self.tokens.entry(account.clone()) {
            let access_token_data = entry.get();
            if !access_token_data.is_expired() {
                tracing::trace!("Using stored access token");
                return Ok(access_token_data.access_token.clone());
            }

            tracing::debug!("Evicting expired access token");
            entry.remove();
        }

        tracing::debug!("Fetching access token for an account");
        let access_token_data = fetch_access_token(account, host, send_request).await?;
        let access_token = access_token_data.access_token.clone();
        self.tokens.insert(account.clone(), access_token_data);

        Ok(access_token)
    }

    /// Forget the cached access token for an account, so that the next request obtains a new one.
    pub fn invalidate_token(&mut self, account: &AccountNumber) {
        self.tokens.remove(account);
    }
}

async fn fetch_access_token(
    account_number: &AccountNumber,
    host: &str,
    send_request: &mut http1::SendRequest<Full<Bytes>>,
) -> Result<AccessTokenData, rest::Error> {
    #[derive(serde::Serialize)]
    struct AccessTokenRequest<'a> {
        account_number: &'a AccountNumber,
    }
    let body = AccessTokenRequest { account_number };

    let body = serde_json::to_vec(&body)?;
    let request = hyper_request_json_bytes(host, ACCESS_TOKEN_PATH, Method::POST, body)?;
    let response = send_request.send_request(request).await?;

    if !response.status().is_success() {
        let Err(e) = handle_error_response(response).await;
        return Err(e);
    }

    let body = response.into_body().collect().await?;
    let access_token_data = serde_json::from_slice(&body.to_bytes())?;

    Ok(access_token_data)
}
