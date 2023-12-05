use std::time::Duration;

use futures::join;
use mullvad_api::{
    self,
    rest::{Error, MullvadRestHandle, RequestServiceHandle},
};
use mullvad_types::location::{AmIMullvad, GeoIpLocation};
use once_cell::sync::Lazy;
use talpid_core::{
    future_retry::{retry_future, ExponentialBackoff, Jittered},
    mpsc::Sender,
};
use talpid_types::ErrorExt;
use tokio::task::JoinHandle;

use crate::{DaemonEventSender, InternalDaemonEvent};

// Define the Mullvad connection checking api endpoint.
//
// In a development build the host name for the connection checking endpoint can
// be overriden by defining the env variable `MULLVAD_CONNCHECK_HOST`.
//
// If `MULLVAD_CONNCHECK_HOST` is set when running `mullvad-daemon` in a
// production build, a warning will be logged and the env variable *won´t* have
// any effect on the api call. The default host name `am.i.mullvad.net` will
// always be used in release mode.
static MULLVAD_CONNCHECK_HOST: Lazy<String> = Lazy::new(|| {
    const DEFAULT_CONNCHECK_HOST: &str = "am.i.mullvad.net";
    let conncheck_host_var = std::env::var("MULLVAD_CONNCHECK_HOST").ok();
    let host = if cfg!(feature = "api-override") {
        match conncheck_host_var.as_deref() {
            Some(host) => {
                log::debug!("Overriding conncheck endpoint. Using {}", &host);
                host
            }
            None => DEFAULT_CONNCHECK_HOST,
        }
    } else {
        if conncheck_host_var.is_some() {
            log::warn!("These variables are ignored in production builds: MULLVAD_CONNCHECK_HOST");
        };
        DEFAULT_CONNCHECK_HOST
    };
    host.to_string()
});

const LOCATION_RETRY_STRATEGY: Jittered<ExponentialBackoff> =
    Jittered::jitter(ExponentialBackoff::new(Duration::from_secs(1), 4));

/// Handler for request to am.i.mullvad.net
pub struct GeoIpHandler {
    /// Unique ID for each request, used to verify
    pub request_id: usize,
    /// Handle to abort any in-flight request.
    request_handle: Option<JoinHandle<()>>,
}

impl GeoIpHandler {
    pub fn new() -> Self {
        Self {
            request_id: 0,
            request_handle: None,
        }
    }

    /// Send geographical location request to am.i.mullvad.net and send the response as an
    /// [`InternalDaemonEvent::LocationEvent`].
    pub fn send_geo_location_request(
        &mut self,
        use_ipv6: bool,
        api_handle: MullvadRestHandle,
        daemon_event_sender: DaemonEventSender,
    ) {
        // Increment request ID
        self.request_id = self.request_id.wrapping_add(1);
        let request_id_copy = self.request_id;

        self.abort_current_request();
        self.request_handle = Some(tokio::spawn(async move {
            if let Ok(merged_location) = get_geo_location_with_retry(api_handle, use_ipv6).await {
                let _ = daemon_event_sender.send(InternalDaemonEvent::LocationEvent((
                    request_id_copy,
                    merged_location,
                )));
            }
        }));
    }

    /// Abort any ongoing call to am.i.mullvad.net
    pub fn abort_current_request(&mut self) {
        if let Some(handle) = self.request_handle.take() {
            handle.abort();
        };
    }
}

/// Fetch the current `GeoIpLocation` with retrys
pub async fn get_geo_location_with_retry(
    api_handle: MullvadRestHandle,
    use_ipv6: bool,
) -> Result<GeoIpLocation, Error> {
    log::debug!("Fetching GeoIpLocation");
    let rest_service = api_handle.service();
    retry_future(
        move || send_location_request(rest_service.clone(), use_ipv6),
        move |result| match result {
            Err(error) if error.is_network_error() => {
                !api_handle.availability.get_state().is_offline()
            }
            _ => false,
        },
        LOCATION_RETRY_STRATEGY,
    )
    .await
}

async fn send_location_request(
    request_sender: RequestServiceHandle,
    use_ipv6: bool,
) -> Result<GeoIpLocation, Error> {
    let v4_sender = request_sender.clone();
    let v4_future = async move {
        let uri_v4 = format!("https://ipv4.{}/json", *MULLVAD_CONNCHECK_HOST);
        let location = send_location_request_internal(&uri_v4, v4_sender).await?;
        Ok::<GeoIpLocation, Error>(GeoIpLocation::from(location))
    };
    let v6_sender = request_sender.clone();
    let v6_future = async move {
        if use_ipv6 {
            let uri_v6 = format!("https://ipv6.{}/json", *MULLVAD_CONNCHECK_HOST);
            let location = send_location_request_internal(&uri_v6, v6_sender).await;
            Some(location.map(GeoIpLocation::from))
        } else {
            None
        }
    };

    let (v4_result, v6_result) = join!(v4_future, v6_future);

    match (v4_result, v6_result) {
        (Ok(mut v4), Some(Ok(v6))) => {
            v4.ipv6 = v6.ipv6;
            v4.mullvad_exit_ip = v4.mullvad_exit_ip && v6.mullvad_exit_ip;
            Ok(v4)
        }
        (Ok(v4), None) => Ok(v4),
        (Ok(v4), Some(Err(e))) => {
            log_network_error(e, "IPv6");
            Ok(v4)
        }
        (Err(e), Some(Ok(v6))) => {
            log_network_error(e, "IPv4");
            Ok(v6)
        }
        (Err(e_v4), _) => Err(e_v4),
    }
}

async fn send_location_request_internal(
    uri: &str,
    service: RequestServiceHandle,
) -> Result<AmIMullvad, Error> {
    let future_service = service.clone();
    let request = mullvad_api::rest::Request::get(uri)?;
    future_service.request(request).await?.deserialize().await
}

fn log_network_error(err: Error, version: &'static str) {
    if !err.is_offline() {
        let err_message = &format!("Unable to fetch {version} GeoIP location");
        log::debug!("{}", err.display_chain_with_msg(err_message));
    }
}
