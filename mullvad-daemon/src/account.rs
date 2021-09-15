use chrono::{DateTime, Utc};
use futures::future::{abortable, AbortHandle};
use mullvad_rpc::{
    availability::ApiAvailabilityHandle,
    rest::{self, MullvadRestHandle},
    AccountsProxy,
};
use mullvad_types::account::{AccountToken, VoucherSubmission};
use std::time::Duration;
use talpid_core::future_retry::{retry_future_with_backoff, ExponentialBackoff, Jittered};

const RETRY_INTERVAL_INITIAL: Duration = Duration::from_secs(4);
const RETRY_INTERVAL_FACTOR: u32 = 5;
const RETRY_INTERVAL_MAX: Duration = Duration::from_secs(24 * 60 * 60);


pub struct Account(());

#[derive(Clone)]
pub struct AccountHandle {
    api_availability: ApiAvailabilityHandle,
    initial_check_abort_handle: AbortHandle,
    pub proxy: AccountsProxy,
}

impl AccountHandle {
    pub async fn check_expiry(&self, token: AccountToken) -> Result<DateTime<Utc>, rest::Error> {
        let result = self.proxy.get_expiry(token).await;
        if handle_expiry_result_inner(&result, &self.api_availability) {
            self.initial_check_abort_handle.abort();
        }
        result
    }

    pub async fn submit_voucher(
        &mut self,
        account_token: AccountToken,
        voucher: String,
    ) -> Result<VoucherSubmission, rest::Error> {
        let result = self.proxy.submit_voucher(account_token, voucher).await;
        if result.is_ok() {
            self.initial_check_abort_handle.abort();
            self.api_availability.resume();
        }
        result
    }
}

impl Account {
    pub fn new(
        runtime: tokio::runtime::Handle,
        rpc_handle: MullvadRestHandle,
        token: Option<String>,
        api_availability: ApiAvailabilityHandle,
    ) -> AccountHandle {
        let accounts_proxy = AccountsProxy::new(rpc_handle);
        api_availability.pause();

        let api_availability_copy = api_availability.clone();
        let accounts_proxy_copy = accounts_proxy.clone();

        let (future, initial_check_abort_handle) = abortable(async move {
            let token = if let Some(token) = token {
                token
            } else {
                api_availability.pause();
                return;
            };

            let retry_strategy = Jittered::jitter(
                ExponentialBackoff::new(RETRY_INTERVAL_INITIAL, RETRY_INTERVAL_FACTOR)
                    .max_delay(RETRY_INTERVAL_MAX),
            );
            let future_generator = move || {
                let wait_online = api_availability.wait_online();
                let expiry_fut = accounts_proxy.get_expiry(token.clone());
                let api_availability_copy = api_availability.clone();
                async move {
                    let _ = wait_online.await;
                    handle_expiry_result_inner(&expiry_fut.await, &api_availability_copy)
                }
            };
            let should_retry = move |state_was_updated: &bool| -> bool { !*state_was_updated };
            let retry_future =
                retry_future_with_backoff(future_generator, should_retry, retry_strategy);
            retry_future.await;
        });
        runtime.spawn(future);

        AccountHandle {
            api_availability: api_availability_copy,
            initial_check_abort_handle,
            proxy: accounts_proxy_copy,
        }
    }
}

fn handle_expiry_result_inner(
    result: &Result<chrono::DateTime<chrono::Utc>, mullvad_rpc::rest::Error>,
    api_availability: &ApiAvailabilityHandle,
) -> bool {
    match result {
        Ok(_expiry) if *_expiry >= chrono::Utc::now() => {
            api_availability.resume();
            true
        }
        Ok(_expiry) => {
            api_availability.pause();
            true
        }
        Err(mullvad_rpc::rest::Error::ApiError(_status, code)) => {
            if code == mullvad_rpc::INVALID_ACCOUNT || code == mullvad_rpc::INVALID_AUTH {
                api_availability.pause();
                return true;
            }
            false
        }
        Err(_) => false,
    }
}
