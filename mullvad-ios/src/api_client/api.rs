use mullvad_api::{
    rest::{self, MullvadRestHandle},
    ApiProxy,
};

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    response::SwiftMullvadApiResponse,
    SwiftApiContext,
};

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_get_addresses(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_get_addresses_inner(api_context.rest_handle()).await {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_api_get_addresses_inner(
    rest_client: MullvadRestHandle,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = ApiProxy::new(rest_client);
    let response = api.get_api_addrs_response().await?;

    SwiftMullvadApiResponse::with_body(response).await
}
