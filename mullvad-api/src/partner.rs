//! Partner API client - create, top-up and delete accounts.

use std::borrow::Borrow;

use crate::{
    ApiEndpoint,
    proxy::ApiConnectionMode,
    rest::{self, MullvadRestHandle},
};

use http::StatusCode;
use mullvad_types::account::AccountNumber;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct PartnerApiClient {
    /// Partner API.
    partner: MullvadRestHandle,
}

impl PartnerApiClient {
    /// mullvad.net
    pub fn mullvad(api_token: String) -> Self {
        let partner = ApiEndpoint::mullvad_partner();
        Self::new(api_token, partner)
    }

    /// stagemole.eu
    pub fn stagemole(api_token: String) -> Self {
        let partner = ApiEndpoint::stagemole_partner();
        Self::new(api_token, partner)
    }

    fn new(api_token: String, partner: ApiEndpoint) -> Self {
        let partner = {
            let runtime = crate::Runtime::new(tokio::runtime::Handle::current(), &partner);
            let mut partner = runtime.mullvad_rest_handle(ApiConnectionMode::Direct);
            // NOTE: `api_token` is expected to be base64-encoded.
            partner.set_auth_header(&format!("Basic {api_token}"));
            partner
        };
        Self { partner }
    }

    /// Create a new Mullvad account.
    pub async fn create_account(&self) -> Result<AccountNumber, rest::Error> {
        #[derive(serde::Deserialize)]
        struct AccountCreationResponse {
            // this is really just account number ..
            id: AccountNumber,
        }
        let request = self.partner.post("v1/accounts")?;
        let response = request.expected_status(&[StatusCode::CREATED]).await?;
        let account: AccountCreationResponse = response.deserialize().await?;

        Ok(account.id)
    }

    /// Add 1 day of VPN time to `account`.
    pub async fn top_up(&self, account: impl Borrow<AccountNumber>) -> Result<(), rest::Error> {
        let account = account.borrow();
        let body = json!({
            "days": 1
        });
        let request = self
            .partner
            .post_json(&format!("v1/accounts/{account}/extend"), &body)?;
        request.expected_status(&[StatusCode::OK]).await?;
        Ok(())
    }
}
