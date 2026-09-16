use std::sync::Arc;
use std::time::Duration;

use futures::{future::OptionFuture, join};
use mullvad_api::{
    availability::ApiAvailability,
    proxy::ApiConnectionMode,
    rest::{Error, RequestService, RequestServiceHandle},
};
use mullvad_types::location::{AmIMullvad, GeoIpLocation, LocationEventData};
use std::sync::LazyLock;
use talpid_core::mpsc::Sender;
use talpid_future::retry::{ExponentialBackoff, Jittered, retry_future};
use talpid_types::ErrorExt;

use crate::DaemonEventSender;

// Define the Mullvad connection checking api endpoint.
//
// In a development build the host name for the connection checking endpoint can
// be overridden by defining the env variable `MULLVAD_CONNCHECK_HOST`.
//
// If `MULLVAD_CONNCHECK_HOST` is set when running `mullvad-daemon` in a
// production build, a warning will be logged and the env variable *won´t* have
// any effect on the api call. The default host name `am.i.mullvad.net` will
// always be used in release mode.
static MULLVAD_CONNCHECK_HOST: LazyLock<String> = LazyLock::new(|| {
    const DEFAULT_CONNCHECK_HOST: &str = "am.i.mullvad.net";
    let conncheck_host_var = std::env::var("MULLVAD_CONNCHECK_HOST").ok();
    let host = if cfg!(feature = "api-override") {
        match conncheck_host_var.as_deref() {
            Some(host) => {
                log::debug!("Overriding conncheck endpoint. Using {host}");
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

/// Handler for request to am.i.mullvad.net, manages in-flight request and validity of responses.
pub(crate) struct GeoIpHandler {
    /// Unique ID for each request. If the ID attached to the
    /// The [`LocationEventData`] used by [`crate::Daemon::handle_location_event`] to
    /// determine if the location belongs to the current tunnel state.
    pub request_id: usize,
    rest_service_ipv4: RequestServiceHandle,
    rest_service_ipv6: RequestServiceHandle,
    location_sender: DaemonEventSender<LocationEventData>,
}

impl GeoIpHandler {
    pub fn new(
        location_sender: DaemonEventSender<LocationEventData>,
        api_availability: ApiAvailability,
        dns_resolver: impl DnsResolver + Clone,
    ) -> Self {
        // Create one rest service per ipv4/v6 conncheck host.
        let [rest_service_ipv4, rest_service_ipv6] = [
            format!("ipv4.{}", *MULLVAD_CONNCHECK_HOST),
            format!("ipv6.{}", *MULLVAD_CONNCHECK_HOST),
        ]
        .map(|host| {
            RequestService::spawn(
                host,
                api_availability.clone(),
                ApiConnectionMode::Direct.into_provider(),
                Arc::new(dns_resolver.clone()),
                #[cfg(target_os = "android")]
                None,
                #[cfg(any(feature = "api-override", test))]
                false,
            )
        });

        Self {
            request_id: 0,
            rest_service_ipv4,
            rest_service_ipv6,
            location_sender,
        }
    }

    /// Send a location request to am.i.mullvad.net. When it arrives, send an
    /// [`LocationEventData`], which triggers an update of the current
    /// tunnel state with the `ipv4` and/or `ipv6` fields filled in.
    pub fn send_geo_location_request(&mut self, use_ipv6: bool) {
        // Increment request ID
        self.request_id = self.request_id.wrapping_add(1);

        self.abort_current_request();

        let request_id = self.request_id;
        let rest_service_ipv4 = self.rest_service_ipv4.clone();
        let rest_service_ipv6 = use_ipv6.then(|| self.rest_service_ipv6.clone());
        let location_sender = self.location_sender.clone();
        tokio::spawn(async move {
            if let Ok(location) =
                get_geo_location_with_retry(rest_service_ipv4, rest_service_ipv6).await
            {
                let _ = location_sender.send(LocationEventData {
                    request_id,
                    location,
                });
            }
        });
    }

    /// Abort any ongoing call to am.i.mullvad.net
    pub fn abort_current_request(&mut self) {
        self.rest_service_ipv4.reset();
        self.rest_service_ipv6.reset();
    }
}

/// Fetch the current `GeoIpLocation` from am.i.mullvad.net. Handles retries on network errors.
async fn get_geo_location_with_retry(
    rest_service_ipv4: RequestServiceHandle,
    rest_service_ipv6: Option<RequestServiceHandle>,
) -> Result<GeoIpLocation, Error> {
    log::debug!("Fetching GeoIpLocation");
    retry_future(
        async move || send_location_request(&rest_service_ipv4, rest_service_ipv6.as_ref()).await,
        move |result| match result {
            Err(error) => error.is_network_error(),
            _ => false,
        },
        LOCATION_RETRY_STRATEGY,
    )
    .await
}

async fn send_location_request(
    rest_service_ipv4: &RequestServiceHandle,
    rest_service_ipv6: Option<&RequestServiceHandle>,
) -> Result<GeoIpLocation, Error> {
    let send_request = async |service: &RequestServiceHandle| {
        let response = service.request().get("json")?.await?;
        let response: AmIMullvad = response.deserialize().await?;
        Ok::<GeoIpLocation, Error>(response.into())
    };

    let v4_future = send_request(rest_service_ipv4);
    let v6_future = OptionFuture::from(rest_service_ipv6.map(send_request));
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

fn log_network_error(err: Error, version: &'static str) {
    if !err.is_offline() {
        let err_message = &format!("Unable to fetch {version} GeoIP location");
        log::debug!("{}", err.display_chain_with_msg(err_message));
    }
}
