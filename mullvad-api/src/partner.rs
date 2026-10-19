//! Partner API client - create, top-up and delete accounts.

use std::borrow::Borrow;

use crate::{
    ApiEndpoint,
    proxy::ApiConnectionMode,
    rest::{self, MullvadRestHandle},
};

use http::{StatusCode, header};
use mullvad_types::account::AccountNumber;
use safelog::Sensitive;
use serde_json::json;

const PARTNER_API_PREFIX: &str = "v1/accounts";

#[derive(Debug, Clone)]
pub struct PartnerApiClient {
    /// Partner API.
    partner: MullvadRestHandle,
    /// Partner API token for authentication.
    api_token: PartnerApiToken,
}

/// An un-printable Partner API token. Used to authenticate against the partner API. See
/// [PartnerApiClient].
pub type PartnerApiToken = Sensitive<String>;

impl PartnerApiClient {
    /// mullvad.net
    pub fn mullvad(api_token: PartnerApiToken) -> Self {
        let partner = ApiEndpoint::mullvad_partner();
        Self::new(api_token, partner)
    }

    /// stagemole.eu
    pub fn stagemole(api_token: PartnerApiToken) -> Self {
        // NOTE: Calling `stagemole_partner` is deliberate here, and does not risk mixing up
        // production and staging environments.
        #[expect(clippy::disallowed_methods)]
        let partner = ApiEndpoint::stagemole_partner();
        Self::new(api_token, partner)
    }

    fn new(api_token: PartnerApiToken, partner: ApiEndpoint) -> Self {
        let partner = {
            let runtime = crate::Runtime::new(tokio::runtime::Handle::current(), &partner);
            runtime.mullvad_rest_handle(ApiConnectionMode::Direct)
        };
        Self { partner, api_token }
    }

    /// Create a new Mullvad account.
    pub async fn create_account(&self) -> Result<AccountNumber, rest::Error> {
        #[derive(serde::Deserialize)]
        struct AccountCreationResponse {
            // this is really just account number ..
            id: AccountNumber,
        }
        let request = self
            .partner
            .post(PARTNER_API_PREFIX)?
            .header(header::AUTHORIZATION, &self.api_token)?;
        let response = request.expected_status(&[StatusCode::CREATED]).await?;
        let account: AccountCreationResponse = response.deserialize().await?;

        Ok(account.id)
    }

    /// Add requested number of days of VPN time to `account`.
    pub async fn add_time(
        &self,
        account: impl Borrow<AccountNumber>,
        days: u32,
    ) -> Result<(), rest::Error> {
        let account = account.borrow();
        let body = json!({
            "days": days
        });
        let request = self
            .partner
            .post_json(&format!("{PARTNER_API_PREFIX}/{account}/extend"), &body)?
            .header(header::AUTHORIZATION, &self.api_token)?;
        request.expected_status(&[StatusCode::OK]).await?;
        Ok(())
    }
}
