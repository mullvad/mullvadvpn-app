use mullvad_api::{rest::MullvadRestHandle, ApiProxy};

use super::{
    completion::{CompletionCookie, SwiftCompletionHandler},
    response::SwiftMullvadApiResponse,
    Error, SwiftApiContext,
};

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_get_addresses(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
) {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
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

async fn mullvad_api_get_addresses_inner(
    rest_client: MullvadRestHandle,
) -> Result<SwiftMullvadApiResponse, Error> {
    let api = ApiProxy::new(rest_client);
    let response = api.get_api_addrs_response().await.map_err(Error::Rest)?;

    SwiftMullvadApiResponse::with_body(response).await
}
