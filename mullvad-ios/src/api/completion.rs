use std::sync::{Arc, Mutex};

use super::response::SwiftMullvadApiResponse;

extern "C" {
    pub fn mullvad_api_completion_finish(
        response: SwiftMullvadApiResponse,
        completion_cookie: CompletionCookie,
    );
}

#[repr(C)]
pub struct CompletionCookie(pub *mut std::ffi::c_void);
unsafe impl Send for CompletionCookie {}

#[derive(Clone)]
pub struct SwiftCompletionHandler {
    inner: Arc<Mutex<Option<CompletionCookie>>>,
}

impl SwiftCompletionHandler {
    pub fn new(cookie: CompletionCookie) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(cookie))),
        }
    }

    pub fn finish(&self, response: SwiftMullvadApiResponse) {
        let Ok(mut maybe_cookie) = self.inner.lock() else {
            log::error!("Response handler panicked");
            return;
        };

        let Some(cookie) = maybe_cookie.take() else {
            return;
        };

        unsafe { mullvad_api_completion_finish(response, cookie) };
    }
}
