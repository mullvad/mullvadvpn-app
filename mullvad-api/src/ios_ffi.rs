use std::{ffi::CString, net::SocketAddr, ptr, sync::Arc};

use crate::{
    rest::{self, MullvadRestHandle},
    AccountsProxy, DevicesProxy,
};

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum MullvadApiErrorKind {
    NoError = 0,
    StringParsing = -1,
    SocketAddressParsing = -2,
    AsyncRuntimeInitialization = -3,
    BadResponse = -4,
    BufferTooSmall = -5,
}

/// MullvadApiErrorKind contains a description and an error kind. If the error kind is
/// `MullvadApiErrorKind` is NoError, the pointer will be nil.
#[repr(C)]
pub struct MullvadApiError {
    description: *mut i8,
    kind: MullvadApiErrorKind,
}

impl MullvadApiError {
    fn new(kind: MullvadApiErrorKind, error: &dyn std::error::Error) -> Self {
        let description = CString::new(format!("{error:?}")).unwrap_or_default();
        Self {
            description: description.into_raw(),
            kind,
        }
    }

    fn api_err(error: &rest::Error) -> Self {
        Self::new(MullvadApiErrorKind::BadResponse, error)
    }

    fn with_str(kind: MullvadApiErrorKind, description: &str) -> Self {
        let description = CString::new(description).unwrap_or_default();
        Self {
            description: description.into_raw(),
            kind,
        }
    }

    fn ok() -> MullvadApiError {
        Self {
            description: CString::new("").unwrap().into_raw(),
            kind: MullvadApiErrorKind::NoError,
        }
    }

    fn drop(self) {
        let _ = unsafe { CString::from_raw(self.description) };
    }
}

/// IosMullvadApiClient is an FFI interface to our `mullvad-api`. It is a thread-safe to accessing
/// our API.
#[derive(Clone)]
#[repr(C)]
pub struct IosMullvadApiClient {
    ptr: *const IosApiClientContext,
}

impl IosMullvadApiClient {
    fn new(context: IosApiClientContext) -> Self {
        let sync_context = Arc::new(context);
        let ptr = Arc::into_raw(sync_context);
        Self { ptr }
    }

    unsafe fn from_raw(self) -> Arc<IosApiClientContext> {
        unsafe {
            Arc::increment_strong_count(self.ptr);
        }

        Arc::from_raw(self.ptr)
    }
}

struct IosApiClientContext {
    tokio_runtime: tokio::runtime::Runtime,
    api_runtime: crate::Runtime,
    api_hostname: String,
}

impl IosApiClientContext {
    fn rest_handle(self: Arc<Self>) -> MullvadRestHandle {
        self.tokio_runtime.block_on(
            self.api_runtime
                .static_mullvad_rest_handle(self.api_hostname.clone()),
        )
    }

    fn devices_proxy(self: Arc<Self>) -> DevicesProxy {
        crate::DevicesProxy::new(self.rest_handle())
    }

    fn accounts_proxy(self: Arc<Self>) -> AccountsProxy {
        crate::AccountsProxy::new(self.rest_handle())
    }

    fn tokio_handle(self: &Arc<Self>) -> tokio::runtime::Handle {
        self.tokio_runtime.handle().clone()
    }
}

/// Paramters:
/// `api_address`: pointer to UTF-8 string containing a socket address representation
/// ("143.32.4.32:9090"), the port is mandatory.
///
/// `api_address_len`: size of the API address string
#[no_mangle]
pub extern "C" fn mullvad_api_initialize_api_runtime(
    context_ptr: *mut IosMullvadApiClient,
    api_address_ptr: *const u8,
    api_address_len: usize,
    hostname: *const u8,
    hostname_len: usize,
) -> MullvadApiError {
    let Some(addr_str) = (unsafe { string_from_raw_ptr(api_address_ptr, api_address_len) }) else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::StringParsing,
            "Failed to parse API socket address string",
        );
    };
    let Some(api_hostname) = (unsafe { string_from_raw_ptr(hostname, hostname_len) }) else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::StringParsing,
            "Failed to parse API host name",
        );
    };

    let Ok(api_address): Result<SocketAddr, _> = addr_str.parse() else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::SocketAddressParsing,
            "Failed to parse API socket address",
        );
    };

    let mut runtime_builder = tokio::runtime::Builder::new_multi_thread();

    runtime_builder.worker_threads(2).enable_all();
    let tokio_runtime = match runtime_builder.build() {
        Ok(runtime) => runtime,
        Err(err) => {
            return MullvadApiError::new(MullvadApiErrorKind::AsyncRuntimeInitialization, &err);
        }
    };

    // It is imperative that the REST runtime is created within an async context, otherwise
    // ApiAvailability panics.
    let api_runtime = tokio_runtime.block_on(async {
        crate::Runtime::with_static_addr(tokio_runtime.handle().clone(), api_address)
    });

    let ios_context = IosApiClientContext {
        tokio_runtime,
        api_runtime,
        api_hostname,
    };

    let context = IosMullvadApiClient::new(ios_context);

    unsafe {
        std::ptr::write(context_ptr, context);
    }

    MullvadApiError::ok()
}

#[no_mangle]
pub extern "C" fn mullvad_api_remove_all_devices(
    context: IosMullvadApiClient,
    account_str_ptr: *const u8,
    account_str_len: usize,
) -> MullvadApiError {
    let ctx = unsafe { context.from_raw() };
    let Some(account) = (unsafe { string_from_raw_ptr(account_str_ptr, account_str_len) }) else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::StringParsing,
            "Failed to parse account number",
        );
    };

    let runtime = ctx.tokio_handle();
    let device_proxy = ctx.devices_proxy();
    let result = runtime.block_on(async move {
        let devices = device_proxy.list(account.clone()).await?;
        for device in devices {
            device_proxy.remove(account.clone(), device.id).await?;
        }
        Result::<_, rest::Error>::Ok(())
    });

    match result {
        Ok(()) => MullvadApiError::ok(),
        Err(err) => MullvadApiError::api_err(&err),
    }
}

#[no_mangle]
pub extern "C" fn mullvad_api_get_expiry(
    context: IosMullvadApiClient,
    account_str_ptr: *const u8,
    account_str_len: usize,
    expiry_unix_timestamp: *mut i64,
) -> MullvadApiError {
    let Some(account) = (unsafe { string_from_raw_ptr(account_str_ptr, account_str_len) }) else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::StringParsing,
            "Failed to parse account number",
        );
    };

    let ctx = unsafe { context.from_raw() };
    let runtime = ctx.tokio_handle();

    let account_proxy = ctx.accounts_proxy();
    let result: Result<_, rest::Error> = runtime.block_on(async move {
        let expiry = account_proxy.get_data(account).await?.expiry;
        let expiry_timestamp = expiry.timestamp();

        Ok(expiry_timestamp)
    });

    match result {
        Ok(expiry) => {
            // SAFETY: It is assumed that expiry_timestamp is a valid pointer to a `libc::timespec`
            unsafe {
                std::ptr::write(expiry_unix_timestamp, expiry);
            }
            MullvadApiError::ok()
        }
        Err(err) => MullvadApiError::api_err(&err),
    }
}

/// Args:
/// context: `IosApiContext`
/// public_key: a pointer to a valid 32 byte array representing a WireGuard public key
#[no_mangle]
pub extern "C" fn mullvad_api_add_device(
    context: IosMullvadApiClient,
    account_str_ptr: *const u8,
    account_str_len: usize,
    public_key_ptr: *const u8,
) -> MullvadApiError {
    let Some(account) = (unsafe { string_from_raw_ptr(account_str_ptr, account_str_len) }) else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::StringParsing,
            "Failed to parse account number",
        );
    };

    let public_key_bytes: [u8; 32] = unsafe { std::ptr::read(public_key_ptr as *const _) };
    let public_key = public_key_bytes.into();

    let ctx = unsafe { context.from_raw() };
    let runtime = ctx.tokio_handle();

    let devices_proxy = ctx.devices_proxy();

    let result: Result<_, rest::Error> = runtime.block_on(async move {
        let (new_device, _) = devices_proxy.create(account, public_key).await?;
        Ok(new_device)
    });

    match result {
        Ok(_result) => MullvadApiError::ok(),
        Err(err) => MullvadApiError::api_err(&err),
    }
}

/// Args:
/// context: `IosApiContext`
/// account_str_ptr: A pointer to a byte buffer large enough to contain a valid account number
/// string.
/// account_str_len: A pointer to an unsigned pointer-sized integer specifying the length of the
/// input buffer. If the buffer is big enough and a new account is created, it will contain the
/// amount of bytes that were written to the buffer.
#[no_mangle]
pub extern "C" fn mullvad_api_create_account(
    context: IosMullvadApiClient,
    account_str_ptr: *mut u8,
    account_str_len: *mut usize,
) -> MullvadApiError {
    let ctx = unsafe { context.from_raw() };
    let runtime = ctx.tokio_handle();
    let buffer_len = unsafe { ptr::read(account_str_len) };

    let accounts_proxy = ctx.accounts_proxy();

    let result: Result<_, rest::Error> = runtime.block_on(async move {
        let new_account = accounts_proxy.create_account().await?;
        Ok(new_account)
    });

    match result {
        Ok(new_account) => {
            let new_account_bytes = new_account.into_bytes();
            if new_account_bytes.len() > buffer_len {
                return MullvadApiError::with_str(
                    MullvadApiErrorKind::BufferTooSmall,
                    "Buffer for account number is too small",
                );
            }
            unsafe {
                ptr::copy(
                    new_account_bytes.as_ptr(),
                    account_str_ptr,
                    new_account_bytes.len(),
                );
            }

            MullvadApiError::ok()
        }
        Err(err) => MullvadApiError::api_err(&err),
    }
}

/// Args:
/// context: `IosApiContext`
/// public_key: a pointer to a valid 32 byte array representing a WireGuard public key
#[no_mangle]
pub extern "C" fn mullvad_api_delete_account(
    context: IosMullvadApiClient,
    account_str_ptr: *const u8,
    account_str_len: usize,
) -> MullvadApiError {
    let ctx = unsafe { context.from_raw() };
    let runtime = ctx.tokio_handle();

    let Some(account) = (unsafe { string_from_raw_ptr(account_str_ptr, account_str_len) }) else {
        return MullvadApiError::with_str(
            MullvadApiErrorKind::StringParsing,
            "Failed to parse account number",
        );
    };

    let accounts_proxy = ctx.accounts_proxy();

    let result: Result<_, rest::Error> = runtime.block_on(async move {
        let new_account = accounts_proxy.delete_account(account).await?;
        Ok(new_account)
    });

    match result {
        Ok(()) => MullvadApiError::ok(),
        Err(err) => MullvadApiError::api_err(&err),
    }
}

#[no_mangle]
pub extern "C" fn mullvad_api_runtime_drop(context: IosMullvadApiClient) {
    unsafe { Arc::decrement_strong_count(context.ptr) }
}

/// The return value is only valid for the lifetime of the `ptr` that's passed in
///
/// SAFETY: `ptr` must be valid for `size` bytes
unsafe fn string_from_raw_ptr(ptr: *const u8, size: usize) -> Option<String> {
    let slice = unsafe { std::slice::from_raw_parts(ptr, size) };

    String::from_utf8(slice.to_vec()).ok()
}

#[no_mangle]
pub extern "C" fn mullvad_api_error_drop(error: MullvadApiError) {
    error.drop()
}
