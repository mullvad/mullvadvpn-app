use chrono::{DateTime, Utc};
use futures::future::{abortable, AbortHandle};
use mullvad_rpc::{
    availability::ApiAvailabilityHandle,
    rest::{self, Error as RestError, MullvadRestHandle},
    AccountsProxy,
};
use mullvad_types::account::{AccountToken, VoucherSubmission};
use std::{future::Future, time::Duration};
use talpid_core::future_retry::{
    constant_interval, retry_future, retry_future_n, ExponentialBackoff, Jittered,
};

const RETRY_ACTION_INTERVAL: Duration = Duration::ZERO;
const RETRY_ACTION_MAX_RETRIES: usize = 2;

const RETRY_EXPIRY_CHECK_INTERVAL_INITIAL: Duration = Duration::from_secs(4);
const RETRY_EXPIRY_CHECK_INTERVAL_FACTOR: u32 = 5;
const RETRY_EXPIRY_CHECK_INTERVAL_MAX: Duration = Duration::from_secs(24 * 60 * 60);


pub struct Account(());

#[derive(Clone)]
pub struct AccountHandle {
    api_availability: ApiAvailabilityHandle,
    initial_check_abort_handle: AbortHandle,
    proxy: AccountsProxy,
}

impl AccountHandle {
    pub fn create_account(&self) -> impl Future<Output = Result<AccountToken, rest::Error>> {
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future_n(
            move || proxy.create_account(),
            move |result| Self::should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
    }

    pub fn get_www_auth_token(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<String, rest::Error>> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future_n(
            move || proxy.get_www_auth_token(account.clone()),
            move |result| Self::should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
    }

    pub async fn check_expiry(&self, token: AccountToken) -> Result<DateTime<Utc>, rest::Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future_n(
            move || proxy.get_expiry(token.clone()),
            move |result| Self::should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
        .await;
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
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future_n(
            move || proxy.submit_voucher(account_token.clone(), voucher.clone()),
            move |result| Self::should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
        .await;
        if result.is_ok() {
            self.initial_check_abort_handle.abort();
            self.api_availability.resume_background();
        }
        result
    }

    fn should_retry<T>(result: &Result<T, RestError>, api_handle: &ApiAvailabilityHandle) -> bool {
        match result {
            Err(error) if error.is_network_error() => !api_handle.get_state().is_offline(),
            _ => false,
        }
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
        api_availability.pause_background();

        let api_availability_copy = api_availability.clone();
        let accounts_proxy_copy = accounts_proxy.clone();

        let (future, initial_check_abort_handle) = abortable(async move {
            let token = if let Some(token) = token {
                token
            } else {
                api_availability.pause_background();
                return;
            };

            let retry_strategy = Jittered::jitter(
                ExponentialBackoff::new(
                    RETRY_EXPIRY_CHECK_INTERVAL_INITIAL,
                    RETRY_EXPIRY_CHECK_INTERVAL_FACTOR,
                )
                .max_delay(RETRY_EXPIRY_CHECK_INTERVAL_MAX),
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
            retry_future(future_generator, should_retry, retry_strategy).await;
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
            api_availability.resume_background();
            true
        }
        Ok(_expiry) => {
            api_availability.pause_background();
            true
        }
        Err(mullvad_rpc::rest::Error::ApiError(_status, code)) => {
            if code == mullvad_rpc::INVALID_ACCOUNT || code == mullvad_rpc::INVALID_AUTH {
                api_availability.pause_background();
                return true;
            }
            false
        }
        Err(_) => false,
    }
}
