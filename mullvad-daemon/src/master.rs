use chrono::DateTime;
use chrono::offset::Utc;
use jsonrpc_client_http::{Error as HttpError, HttpCore, HttpHandle};

use mullvad_types::account::AccountToken;

static MASTER_API_URI: &str = "https://api.mullvad.net/rpc/";

pub fn create_account_proxy() -> Result<AccountsProxy<HttpError, HttpHandle>, HttpError> {
    let core = HttpCore::standalone()?;
    let transport = core.handle(MASTER_API_URI)?;
    Ok(AccountsProxy::new(transport))
}

jsonrpc_client!(pub struct AccountsProxy {
    pub fn get_expiry(&mut self, account_token: AccountToken) -> RpcRequest<DateTime<Utc>>;
});
