#![cfg(target_os = "ios")]
mod encrypted_dns_proxy;
mod ephemeral_peer_proxy;
mod shadowsocks_proxy;
pub mod tunnel_obfuscator_proxy;

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[unsafe(no_mangle)]
pub static CONFIG_SERVICE_PORT: u16 = talpid_tunnel_config_client::CONFIG_SERVICE_PORT;

mod ios {
    use std::sync::OnceLock;
    use tokio::runtime::{Builder, Handle, Runtime};

    static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

    pub fn mullvad_ios_runtime() -> Result<Handle, String> {
        match RUNTIME.get_or_init(|| {
            Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|error| ToString::to_string(&error))
        }) {
            Ok(runtime) => Ok(runtime.handle().clone()),
            Err(error) => Err(error.clone()),
        }
    }
}

use ios::*;
use mullvad_api::{
    proxy::{ApiConnectionMode, StaticConnectionModeProvider},
    rest::{MullvadRestHandle, Response},
    ApiEndpoint, ApiProxy, Runtime,
};
use std::{
    ffi::{CStr, CString},
    net::Incoming,
    ptr::{null, null_mut},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
    u8,
};

extern "C" {
    pub fn completion_finish(
        response: SwiftMullvadApiResponse,
        completion_cookie: CompletionCookie,
    );
}

#[repr(C)]
pub struct CompletionCookie(*mut std::ffi::c_void);
unsafe impl Send for CompletionCookie {}

#[derive(Clone)]
pub struct SwiftCompletionHandler {
    inner: Arc<Mutex<Option<CompletionCookie>>>,
}

impl SwiftCompletionHandler {
    fn new(cookie: CompletionCookie) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(cookie))),
        }
    }

    fn finish(&self, response: SwiftMullvadApiResponse) {
        let Ok(mut maybe_cookie) = self.inner.lock() else {
            log::error!("Response handler panicked");
            return;
        };

        let Some(cookie) = maybe_cookie.take() else {
            return;
        };

        unsafe { completion_finish(response, cookie) };
    }
}

#[repr(C)]
pub struct SwiftApiContext(*const ApiContext);
impl SwiftApiContext {
    pub fn new(context: ApiContext) -> SwiftApiContext {
        SwiftApiContext(Arc::into_raw(Arc::new(context)))
    }

    pub unsafe fn to_rust_context(self) -> Arc<ApiContext> {
        Arc::increment_strong_count(self.0);
        Arc::from_raw(self.0)
    }
}

#[repr(C)]
pub struct SwiftMullvadApiResponse {
    body: *mut u8,
    body_size: usize,
    status_code: u16,
    error_description: *mut u8,
    success: bool,
}
impl SwiftMullvadApiResponse {
    async fn with_body(response: Response<hyper::body::Incoming>) -> Result<Self, Error> {
        let status_code: u16 = response.status().into();
        let body: Vec<u8> = response.body().await.map_err(Error::Rest)?;

        let body_size = body.len();
        let body = body.into_boxed_slice();

        Ok(Self {
            body: Box::into_raw(body).cast(),
            body_size,
            status_code,
            error_description: null_mut(),
            success: true
        })
    }

    fn error() -> Self {
        Self {
            body: null_mut(),
            body_size: 0,
            status_code: 0,
            error_description: null_mut(),
            success: false,
        }
    }
}

struct ApiContext {
    api_client: Runtime,
    rest_client: MullvadRestHandle,
}
impl ApiContext {
    pub fn rest_handle(&self) -> MullvadRestHandle {
        self.rest_client.clone()
    }
}

#[no_mangle]
pub extern "C" fn mullvad_api_init_new(host: *const u8, address: *const u8) -> SwiftApiContext {
    let host = unsafe { CStr::from_ptr(host.cast()) };
    let address = unsafe { CStr::from_ptr(address.cast()) };

    let Ok(host) = host.to_str() else {
        return SwiftApiContext(null_mut());
    };

    let Ok(address) = address.to_str() else {
        return SwiftApiContext(null_mut());
    };

    let endpoint = ApiEndpoint {
        host: Some(String::from_str(host).unwrap()),
        address: Some(address.parse().unwrap()),
    };

    let tokio_handle = match crate::mullvad_ios_runtime() {
        Ok(tokio_handle) => tokio_handle,
        Err(err) => {
            log::error!("Failed to obtain a handle to a tokio runtime: {err}");
            return SwiftApiContext(null_mut());
        }
    };

    let api_context = tokio_handle.clone().block_on(async move {
        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.
        let api_client = mullvad_api::Runtime::new(tokio_handle, &endpoint);
        let rest_client = api_client
            .mullvad_rest_handle(StaticConnectionModeProvider::new(ApiConnectionMode::Direct));

        ApiContext {
            api_client,
            rest_client,
        }
    });

    SwiftApiContext::new(api_context)
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_get_addresses(
    api_context: SwiftApiContext,
    completion_cookie: CompletionCookie,
) {
    let completion_handler = SwiftCompletionHandler::new(completion_cookie);

    let Ok(tokio_handle) = mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::error());
        return;
    };

    let api_context = api_context.to_rust_context();

    tokio_handle.clone().spawn(async move {
        match mullvad_api_get_addresses_inner(api_context.rest_handle()).await {
            Ok(response) => completion_handler.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion_handler.finish(SwiftMullvadApiResponse::error());
            }
        }
    });
}

#[derive(Debug)]
enum Error {
    Rest(mullvad_api::rest::Error),
    CString(std::ffi::NulError),
}

async fn mullvad_api_get_addresses_inner(rest_client: MullvadRestHandle) -> Result<SwiftMullvadApiResponse, Error> {
    let api = ApiProxy::new(rest_client);
    let response = api.get_api_addrs_response().await.map_err(Error::Rest)?;

    SwiftMullvadApiResponse::with_body(response).await
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_response_drop(response: SwiftMullvadApiResponse) {
    if !response.body.is_null() {
        let _ = Vec::from_raw_parts(response.body, response.body_size, response.body_size);
    }

    if !response.error_description.is_null() {
        let _ = CStr::from_ptr(response.error_description.cast());
    }
}
