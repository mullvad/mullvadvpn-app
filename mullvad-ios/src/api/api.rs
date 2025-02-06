use mullvad_api::{rest::{self, MullvadRestHandle}, ApiProxy};

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    response::SwiftMullvadApiResponse,
    Error, SwiftApiContext,
};

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

    let api_context = api_context.to_rust_context();

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

    RequestCancelHandle::new(task, completion_handler.clone()).to_swift()
}

async fn mullvad_api_get_addresses_inner(
    rest_client: MullvadRestHandle,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = ApiProxy::new(rest_client);
    let response = api.get_api_addrs_response().await?;

    SwiftMullvadApiResponse::with_body(response).await
}
