use crate::{
    rest,
    rest::{RequestFactory, RequestServiceHandle},
};
use hyper::StatusCode;
use mullvad_types::account::{AccessToken, AccessTokenData, AccountToken};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use talpid_types::ErrorExt;

pub const AUTH_URL_PREFIX: &str = "auth/v1";

#[derive(Clone)]
pub struct AccessTokenProxy {
    service: RequestServiceHandle,
    factory: RequestFactory,
    access_from_account: Arc<Mutex<HashMap<AccountToken, AccessTokenData>>>,
}

impl AccessTokenProxy {
    pub(crate) fn new(service: RequestServiceHandle, factory: RequestFactory) -> Self {
        Self {
            service,
            factory,
            access_from_account: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Obtain access token for an account, requesting a new one from the API if necessary.
    pub async fn get_token(&self, account: &AccountToken) -> Result<AccessToken, rest::Error> {
        let existing_token = {
            self.access_from_account
                .lock()
                .unwrap()
                .get(account.as_str())
                .cloned()
        };
        if let Some(access_token) = existing_token {
            if access_token.is_expired() {
                log::debug!("Replacing expired access token");
                return self.request_new_token(account.clone()).await;
            }
            log::trace!("Using stored access token");
            return Ok(access_token.access_token.clone());
        }
        self.request_new_token(account.clone()).await
    }

    /// Remove an access token if the API response calls for it.
    pub fn check_response<T>(&self, account: &AccessToken, response: &Result<T, rest::Error>) {
        if let Err(rest::Error::ApiError(_status, code)) = response {
            if code == crate::INVALID_ACCESS_TOKEN {
                log::debug!("Dropping invalid access token");
                self.remove_token(account);
            }
        }
    }

    /// Removes a stored access token.
    fn remove_token(&self, account: &AccountToken) -> Option<AccessToken> {
        self.access_from_account
            .lock()
            .unwrap()
            .remove(account)
            .map(|v| v.access_token)
    }

    async fn request_new_token(&self, account: AccountToken) -> Result<AccessToken, rest::Error> {
        log::debug!("Fetching access token for an account");
        let access_token = self
            .fetch_access_token(account.clone())
            .await
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to obtain access token")
                );
                error
            })?;
        self.access_from_account
            .lock()
            .unwrap()
            .insert(account, access_token.clone());
        Ok(access_token.access_token)
    }

    async fn fetch_access_token(
        &self,
        account_token: AccountToken,
    ) -> Result<AccessTokenData, rest::Error> {
        #[derive(serde::Serialize)]
        struct AccessTokenRequest {
            account_number: String,
        }
        let request = AccessTokenRequest {
            account_number: account_token,
        };

        let service = self.service.clone();

        let rest_request = self
            .factory
            .post_json(&format!("{AUTH_URL_PREFIX}/token"), &request)?;
        let response = service.request(rest_request).await?;
        let response = rest::parse_rest_response(response, &[StatusCode::OK]).await?;
        rest::deserialize_body(response).await
    }
}
