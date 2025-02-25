#![cfg(not(target_os = "android"))]
#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use std::{
    ffi::{CStr, CString},
    net::SocketAddr,
    ptr,
    sync::Arc,
};

use crate::{
    proxy::ApiConnectionMode,
    rest::{self, MullvadRestHandle},
    AccountsProxy, ApiEndpoint, DevicesProxy,
};

mod device;
mod error;

pub use error::{MullvadApiError, MullvadApiErrorKind};

#[repr(C)]
pub struct MullvadApiClient {
    ptr: *const FfiClient,
}

impl MullvadApiClient {
    fn new(client: FfiClient) -> Self {
        let arc = Arc::new(client);
        let ptr = Arc::into_raw(arc);
        Self { ptr }
    }

    unsafe fn get_client(&self) -> Arc<FfiClient> {
        // Incrementing before creating an Arc from a pointer. This way multiple threads can use
        // it, and a single thread can decrement it.
        unsafe { Arc::increment_strong_count(self.ptr) };

        unsafe { Arc::from_raw(self.ptr) }
    }

    fn drop(self) {
        if self.ptr.is_null() {
            return;
        }

        let _ = unsafe { Arc::from_raw(self.ptr) };
    }
}

/// A Mullvad API client that can be used via a C FFI.
struct FfiClient {
    tokio_runtime: tokio::runtime::Runtime,
    api_runtime: crate::Runtime,
}

impl FfiClient {
    unsafe fn new(
        api_address_ptr: *const libc::c_char,
        hostname: *const libc::c_char,
        #[cfg(any(feature = "api-override", test))] disable_tls: bool,
    ) -> Result<Self, MullvadApiError> {
        // SAFETY: addr_str must be a valid pointer to a null-terminated string.
        let addr_str = unsafe { string_from_raw_ptr(api_address_ptr)? };
        // SAFETY: api_hostname must be a valid pointer to a null-terminated string.
        let api_hostname = unsafe { string_from_raw_ptr(hostname)? };

        let api_address: SocketAddr = addr_str.parse().map_err(|_| {
            MullvadApiError::with_str(
                MullvadApiErrorKind::SocketAddressParsing,
                "Failed to parse API socket address",
            )
        })?;

        let endpoint = ApiEndpoint {
            host: Some(api_hostname.clone()),
            address: Some(api_address),
            #[cfg(feature = "api-override")]
            force_direct: false,
            #[cfg(any(feature = "api-override", test))]
            disable_tls,
        };

        let mut runtime_builder = tokio::runtime::Builder::new_multi_thread();

        runtime_builder.worker_threads(2).enable_all();
        let tokio_runtime = runtime_builder.build().map_err(|err| {
            MullvadApiError::new(MullvadApiErrorKind::AsyncRuntimeInitialization, &err)
        })?;

        // It is imperative that the REST runtime is created within an async context, otherwise
        // ApiAvailability panics.
        let api_runtime = tokio_runtime
            .block_on(async { crate::Runtime::new(tokio_runtime.handle().clone(), &endpoint) });

        let context = FfiClient {
            tokio_runtime,
            api_runtime,
        };

        Ok(context)
    }

    unsafe fn add_device(
        self: Arc<Self>,
        account_str_ptr: *const libc::c_char,
        public_key_ptr: *const u8,
    ) -> Result<device::MullvadApiDevice, MullvadApiError> {
        // SAFETY: account_str_ptr must be a valid pointer to a null-terminated string.
        let account = unsafe { string_from_raw_ptr(account_str_ptr)? };

        // SAFETY: assuming public_key_ptr is valid for 32 bytes
        let public_key_bytes: [u8; 32] = unsafe { std::ptr::read(public_key_ptr as *const _) };
        let public_key = public_key_bytes.into();

        let runtime = self.tokio_handle();

        let device_proxy = self.device_proxy();

        let device = runtime
            .block_on(async move {
                let (device, _) = device_proxy.create(account, public_key).await?;
                Ok(device)
            })
            .map_err(MullvadApiError::api_err)?;

        Ok(device.into())
    }

    unsafe fn create_account(self: Arc<Self>) -> Result<String, MullvadApiError> {
        let accounts_proxy = self.accounts_proxy();

        self.tokio_handle()
            .block_on(async move {
                let new_account = accounts_proxy.create_account().await?;
                Ok(new_account)
            })
            .map_err(MullvadApiError::api_err)
    }

    unsafe fn get_expiry(
        self: Arc<Self>,
        account_str_ptr: *const libc::c_char,
    ) -> Result<i64, MullvadApiError> {
        // SAFETY: account_str_ptr must be a valid pointer to a null-terminated string.
        let account = unsafe { string_from_raw_ptr(account_str_ptr)? };

        let account_proxy = self.accounts_proxy();
        self.tokio_handle()
            .block_on(async move {
                let expiry_timestamp = account_proxy.get_data(account).await?.expiry.timestamp();
                Ok(expiry_timestamp)
            })
            .map_err(MullvadApiError::api_err)
    }

    unsafe fn remove_all_devices(
        self: Arc<Self>,
        account_str_ptr: *const libc::c_char,
    ) -> Result<(), MullvadApiError> {
        // SAFETY: account_str_ptr must be a valid pointer to a null-terminated string.
        let account = unsafe { string_from_raw_ptr(account_str_ptr)? };

        let runtime = self.tokio_handle();
        let device_proxy = self.device_proxy();
        runtime
            .block_on(async move {
                let devices = device_proxy.list(account.clone()).await?;
                for device in devices {
                    device_proxy.remove(account.clone(), device.id).await?;
                }
                Result::<_, rest::Error>::Ok(())
            })
            .map_err(MullvadApiError::api_err)
    }

    unsafe fn list_devices(
        self: Arc<Self>,
        account_str_ptr: *const libc::c_char,
    ) -> Result<device::MullvadApiDeviceIterator, MullvadApiError> {
        // SAFETY: account_str_ptr must be a valid pointer to a null-terminated string.
        let account = unsafe { string_from_raw_ptr(account_str_ptr)? };

        let runtime = self.tokio_handle();
        let device_proxy = self.device_proxy();

        let devices = runtime
            .block_on(device_proxy.list(account))
            .map_err(MullvadApiError::api_err)?;

        Ok(device::MullvadApiDeviceIterator::new(devices))
    }

    unsafe fn delete_account(
        self: Arc<Self>,
        account_str_ptr: *const libc::c_char,
    ) -> Result<(), MullvadApiError> {
        // SAFETY: account_str_ptr must be a valid pointer to a null-terminated string.
        let account = unsafe { string_from_raw_ptr(account_str_ptr)? };

        let runtime = self.tokio_handle();
        let accounts_proxy = self.accounts_proxy();

        runtime
            .block_on(accounts_proxy.delete_account(account))
            .map_err(MullvadApiError::api_err)
    }

    fn rest_handle(&self) -> MullvadRestHandle {
        self.tokio_handle().block_on(async {
            self.api_runtime
                .mullvad_rest_handle(ApiConnectionMode::Direct.into_provider())
        })
    }

    fn device_proxy(&self) -> DevicesProxy {
        crate::DevicesProxy::new(self.rest_handle())
    }

    fn accounts_proxy(&self) -> AccountsProxy {
        crate::AccountsProxy::new(self.rest_handle())
    }

    fn tokio_handle(&self) -> tokio::runtime::Handle {
        self.tokio_runtime.handle().clone()
    }
}

/// Initializes a Mullvad API client.
///
/// # Safety
///
/// * `client_ptr`: Must be a pointer to that is valid for the length of a `MullvadApiClient`
///   struct.
///
/// * `api_address`: pointer to nul-terminated UTF-8 string containing a socket address
///   representation ("143.32.4.32:9090"), the port is mandatory.
///
/// * `hostname`: pointer to a null-terminated UTF-8 string representing the hostname that will be
///   used for TLS validation.
/// * `disable_tls`: only valid when built for tests, can be ignored when consumed by Swift.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_client_initialize(
    client_ptr: *mut MullvadApiClient,
    api_address_ptr: *const libc::c_char,
    hostname: *const libc::c_char,
    disable_tls: bool,
) -> MullvadApiError {
    #[cfg(not(any(feature = "api-override", test)))]
    if disable_tls {
        log::error!("disable_tls has no effect when mullvad-api is built without api-override");
    }

    match unsafe {
        FfiClient::new(
            api_address_ptr,
            hostname,
            #[cfg(any(feature = "api-override", test))]
            disable_tls,
        )
    } {
        Ok(client) => {
            unsafe {
                std::ptr::write(client_ptr, MullvadApiClient::new(client));
            };
            MullvadApiError::ok()
        }
        Err(err) => err,
    }
}

/// Removes all devices from a given account
///
/// # Safety
///
/// * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
///
/// * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
///   account that will have all of it's devices removed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_remove_all_devices(
    client_ptr: MullvadApiClient,
    account_ptr: *const libc::c_char,
) -> MullvadApiError {
    let client = unsafe { client_ptr.get_client() };
    match unsafe { client.remove_all_devices(account_ptr) } {
        Ok(_) => MullvadApiError::ok(),
        Err(err) => err,
    }
}

/// Removes all devices from a given account
///
/// # Safety
/// * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
///
/// * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
///   account that will have all of it's devices removed.
///
/// * `expiry_unix_timestamp`: a pointer to a signed 64 bit integer. If this function returns no
///   error, the expiry timestamp will be written to this pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_get_expiry(
    client_ptr: MullvadApiClient,
    account_str_ptr: *const libc::c_char,
    expiry_unix_timestamp: *mut i64,
) -> MullvadApiError {
    let client = unsafe { client_ptr.get_client() };
    match unsafe { client.get_expiry(account_str_ptr) } {
        Ok(expiry) => {
            unsafe { ptr::write(expiry_unix_timestamp, expiry) };
            MullvadApiError::ok()
        }
        Err(err) => err,
    }
}

/// Gets a list of all devices associated with the specified account from the API.
///
/// # Safety
///
/// * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
///
/// * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
///   account that will have all of it's devices removed.
///
/// * `device_iter_ptr`: a pointer to a `device::MullvadApiDeviceIterator`. If this function doesn't
///   return an error, the pointer will be initialized with a valid instance of
///   `device::MullvadApiDeviceIterator`, which can be used to iterate through the devices.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_list_devices(
    client_ptr: MullvadApiClient,
    account_str_ptr: *const libc::c_char,
    device_iter_ptr: *mut device::MullvadApiDeviceIterator,
) -> MullvadApiError {
    let client = unsafe { client_ptr.get_client() };
    match unsafe { client.list_devices(account_str_ptr) } {
        Ok(iter) => {
            unsafe { ptr::write(device_iter_ptr, iter) };
            MullvadApiError::ok()
        }
        Err(err) => err,
    }
}

/// Adds a device to the specified account with the specified public key. Note that the device
/// name, associated addresess and UUID are not returned.
///
/// # Safety
///
/// * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
///
/// * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
///   account that will have a device added to ita device added to it.
///
/// * `public_key_ptr`: a pointer to 32 bytes of a WireGuard public key that will be uploaded.
///
/// * `new_device_ptr`: a pointer to enough memory to allocate a `MullvadApiDevice`. If this
///   function doesn't return an error, it will be initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_add_device(
    client_ptr: MullvadApiClient,
    account_str_ptr: *const libc::c_char,
    public_key_ptr: *const u8,
    new_device_ptr: *mut device::MullvadApiDevice,
) -> MullvadApiError {
    // SAFETY: Assuming MullvadApiClient is initialized
    let client = unsafe { client_ptr.get_client() };
    // SAFETY: Asuming `new_device_ptr` is valid.
    match unsafe { client.add_device(account_str_ptr, public_key_ptr) } {
        Ok(device) => {
            // SAFETY: Asuming `new_device_ptr` is valid.
            // SAFETY: Asuming `new_device_ptr` is valid.
            unsafe { ptr::write(new_device_ptr, device) };
            MullvadApiError::ok()
        }
        Err(err) => err,
    }
}

/// Creates a new account.
///
/// # Safety
///
/// * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
///
/// * `account_str_ptr`: If a new account is created successfully, a pointer to an allocated C
///   string containing the new account number will be written to this pointer. It must be freed via
///   `mullvad_api_cstring_drop`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_create_account(
    client_ptr: MullvadApiClient,
    account_str_ptr: *mut *const libc::c_char,
) -> MullvadApiError {
    let client = unsafe { client_ptr.get_client() };
    match unsafe { client.create_account() } {
        Ok(new_account) => {
            let Ok(account) = CString::new(new_account) else {
                return MullvadApiError::with_str(
                    MullvadApiErrorKind::BadResponse,
                    "Account number string c ontained null bytes",
                );
            };

            unsafe { ptr::write(account_str_ptr, account.into_raw()) };
            MullvadApiError::ok()
        }
        Err(err) => err,
    }
}

/// Deletes the specified account.
///
/// # Safety
///
/// * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
///
/// * `account_str_ptr`: Must be a null-terminated string representing the account to be deleted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_delete_account(
    client_ptr: MullvadApiClient,
    account_str_ptr: *const libc::c_char,
) -> MullvadApiError {
    let client = unsafe { client_ptr.get_client() };
    match unsafe { client.delete_account(account_str_ptr) } {
        Ok(_) => MullvadApiError::ok(),
        Err(err) => err,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mullvad_api_client_drop(client: MullvadApiClient) {
    client.drop()
}

/// Deallocates a CString returned by the Mullvad API client.
///
/// # Safety
///
/// `cstr_ptr` must be a pointer to a string allocated by another `mullvad_api` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_cstring_drop(cstr_ptr: *mut libc::c_char) {
    let _ = unsafe { CString::from_raw(cstr_ptr) };
}

/// The return value is only valid for the lifetime of the `ptr` that's passed in
///
/// # Safety
///
/// `ptr` must be valid for `size` bytes
unsafe fn string_from_raw_ptr(ptr: *const libc::c_char) -> Result<String, MullvadApiError> {
    let cstr = unsafe { CStr::from_ptr(ptr) };

    Ok(cstr
        .to_str()
        .map_err(|_| {
            MullvadApiError::with_str(
                MullvadApiErrorKind::StringParsing,
                "Failed to parse UTF-8 string",
            )
        })?
        .to_owned())
}

#[cfg(test)]
mod test {
    use mockito::{Server, ServerGuard};
    use std::{mem::MaybeUninit, net::Ipv4Addr};

    use super::*;
    const STAGING_HOSTNAME: &[u8] = b"api-app.stagemole.eu\0";

    #[test]
    fn test_initialization() {
        let _ = create_client(&SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 1));
    }

    fn create_client(addr: &SocketAddr) -> MullvadApiClient {
        let mut client = MaybeUninit::<MullvadApiClient>::uninit();
        let cstr_address = CString::new(addr.to_string()).unwrap();
        unsafe {
            mullvad_api_client_initialize(
                client.as_mut_ptr(),
                cstr_address.as_ptr().cast(),
                STAGING_HOSTNAME.as_ptr().cast(),
                true,
            )
            .unwrap();
        };
        unsafe { client.assume_init() }
    }

    #[test]
    fn test_create_delete_account() {
        let server = test_server();
        let client = create_client(&server.socket_address());

        let mut account_buf = vec![0 as libc::c_char; 100];
        unsafe { mullvad_api_create_account(client, account_buf.as_mut_ptr().cast()).unwrap() };
    }

    fn test_server() -> ServerGuard {
        let mut server = Server::new();
        let expected_create_account_response = br#"{"id":"085df870-0fc2-47cb-9e8c-cb43c1bdaac0","expiry":"2024-12-11T12:56:32+00:00","max_ports":0,"can_add_ports":false,"max_devices":5,"can_add_devices":true,"number":"6705749539195318"}"#;
        server
            .mock(
                "POST",
                &*("/".to_string() + crate::ACCOUNTS_URL_PREFIX + "/accounts"),
            )
            .with_header("content-type", "application/json")
            .with_status(201)
            .with_body(expected_create_account_response)
            .create();

        server
    }
}
