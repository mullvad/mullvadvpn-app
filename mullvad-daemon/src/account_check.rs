use mullvad_rpc::{availability::ApiAvailabilityHandle, AccountsProxy};
use std::time::Duration;
use talpid_core::future_retry::{retry_future_with_backoff, ExponentialBackoff, Jittered};

const RETRY_INTERVAL_INITIAL: Duration = Duration::from_secs(4);
const RETRY_INTERVAL_FACTOR: u32 = 5;
const RETRY_INTERVAL_MAX: Duration = Duration::from_secs(24 * 60 * 60);


pub struct AccountChecker(());

#[derive(Clone)]
pub struct AccountCheckerHandle {
    api_availability: ApiAvailabilityHandle,
}

impl AccountCheckerHandle {
    pub fn handle_expiry_result(
        &self,
        result: &Result<chrono::DateTime<chrono::Utc>, mullvad_rpc::rest::Error>,
    ) {
        AccountChecker::handle_expiry_result_inner(result, self.api_availability.clone());
    }
}

impl AccountChecker {
    pub fn new(
        runtime: tokio::runtime::Handle,
        token: Option<String>,
        api_availability: ApiAvailabilityHandle,
        accounts_proxy: AccountsProxy,
    ) -> AccountCheckerHandle {
        api_availability.pause();

        let api_availability_copy = api_availability.clone();
        runtime.spawn(async move {
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
                    Self::handle_expiry_result_inner(&expiry_fut.await, api_availability_copy)
                }
            };
            let should_retry = move |state_was_updated: &bool| -> bool { !*state_was_updated };
            let retry_future =
                retry_future_with_backoff(future_generator, should_retry, retry_strategy);
            retry_future.await;
        });

        AccountCheckerHandle {
            api_availability: api_availability_copy,
        }
    }

    fn handle_expiry_result_inner(
        result: &Result<chrono::DateTime<chrono::Utc>, mullvad_rpc::rest::Error>,
        api_availability: ApiAvailabilityHandle,
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
}
