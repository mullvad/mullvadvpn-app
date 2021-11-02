use crate::{
    rest,
    rest::{RequestFactory, RequestServiceHandle},
};
use hyper::{Method, StatusCode};
use mullvad_types::account::{AccessToken, AccessTokenData, AccountToken};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use talpid_types::ErrorExt;

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

    async fn request_new_token(&self, account: AccountToken) -> Result<AccessToken, rest::Error> {
        log::debug!("Fetching access token for an account");
        let access_token = self
            .fetch_access_token(account.clone())
            .await
            .map_err(|error| {
                match &error {
                    rest::Error::ApiError(status, _code) if status == &StatusCode::BAD_REQUEST => {
                        log::error!("Failed to obtain access token: Invalid account");
                    }
                    error => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to obtain access token")
                        );
                    }
                }
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
            account_token: String,
        }
        let request = AccessTokenRequest { account_token };

        let service = self.service.clone();
        let response = rest::send_json_request(
            &self.factory,
            service,
            "auth/token",
            Method::POST,
            &request,
            None,
            &[StatusCode::OK],
        );
        rest::deserialize_body(response.await?).await
    }
}
